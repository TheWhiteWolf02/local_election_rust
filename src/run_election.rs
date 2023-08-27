use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::emmc_client::*;

/* for debugging purposes */
static mut VERBOSE: bool = false;

/* consts */
const WRITETIMEOUT: u64 = 50; // 50 millis
const READTIMEOUT: u64 = 10; // 10 millis
const TERM: u64 = 5000; // 5 secs
const LEADERTIMEOUT: u64 = TERM - READTIMEOUT;

pub(crate) struct Bundle {
    pub(crate) op: Option<fn()>,
}

fn init() -> u64 {
    if unsafe { VERBOSE } {
        println!("init - initializing emmc");
    }

    let server_addresses = ServerAddresses {
        read: SocketAddr::new(IpAddr::from([127, 0, 0, 1]), EMMCPORT),
        write: SocketAddr::new(IpAddr::from([127, 0, 0, 1]), EMMCPORT),
    };
    init_emmc(&server_addresses);

    // Generate a random number
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let millis = (now.as_secs() as i64) * 1000 + (now.subsec_nanos() as i64) / 1_000_000;
    let mut rng: StdRng = SeedableRng::seed_from_u64(millis.try_into().unwrap());
    let id: u64 = rng.gen();
    println!("init - id: {}", id);

    if unsafe { VERBOSE } {
        println!("init - done");
    }

    let mut fp = File::create("/aborted.tmp").expect("Unable to create file");
    fp.write_all(b"0").expect("Unable to write data");

    return id;
}

fn abort() {
    if let Ok(mut fp) = File::create("/aborted.tmp") {
        if let Err(err) = writeln!(fp, "1") {
            eprintln!("Failed to write to aborted.tmp: {}", err);
        }
    } else {
        eprintln!("Failed to create aborted.tmp");
    }

    std::process::exit(1);
}

fn check_read_id(read_id: u64, id: u64) -> u64 {
    if unsafe { VERBOSE } {
        println!("check read id - Checking file");
    }
    /*
    if LEADER {
        if let Ok(mut fp) = File::create("/read_id.tmp") {
            writeln!(
                fp,
                "Read id: {} - This id: {}",
                read_id,
                ID.load(Ordering::SeqCst)
            )
            .ok();
        }
    }
    */
    if unsafe { VERBOSE } {
        println!("check read id - EB id: {} - my id: {}", read_id, id);
    }

    if read_id == id {
        println!("Both read_id and ID is found equal");
        return 0;
    }

    return 1;
}

fn read_from_election_block() -> u64 {
    let server_addresses = ServerAddresses {
        read: SocketAddr::new(IpAddr::from([127, 0, 0, 1]), EMMCPORT),
        write: SocketAddr::new(IpAddr::from([127, 0, 0, 1]), EMMCPORT),
    };
    let read_id: u64 = {
        let read_socket =
            init_read_socket(&server_addresses.read).expect("Unable to re-initialize read socket");
        _read_from_election_block(&read_socket).expect("Error reading from election block")
    };
    return read_id;
}

fn read_from_election_block_caller(done: &Arc<Mutex<AtomicBool>>, id: u64) {
    let read_id = read_from_election_block();

    let res = check_read_id(read_id, id);
    if res != 0 {
        println!("read check failed");
        abort();
    }

    if unsafe { VERBOSE } {
        println!("Read from election block - id checked");
        println!("Read from election block - Done");
    }
    done.lock().unwrap().store(true, Ordering::Relaxed);
}

fn write_to_election_block(new_id: u64) -> u64 {
    let server_addresses = ServerAddresses {
        read: SocketAddr::new(IpAddr::from([127, 0, 0, 1]), EMMCPORT),
        write: SocketAddr::new(IpAddr::from([127, 0, 0, 1]), EMMCPORT),
    };

    let write_id: u64 = {
        let write_socket = init_write_socket(&server_addresses.write)
            .expect("Unable to re-initialize write socket");
        _write_to_election_block(&write_socket, new_id).expect("Error writing to election block")
    };
    return write_id;
}

fn write_and_check_election_block(done: &Arc<Mutex<AtomicBool>>, id: u64) {
    if unsafe { VERBOSE } {
        println!("Writing to election block");
    }

    let res = write_to_election_block(id);
    if res != 0 {
        abort();
    }

    if unsafe { VERBOSE } {
        println!("Reading from election block");
    }

    let read_res = read_from_election_block();

    if unsafe { VERBOSE } {
        println!("Read from election block - read id: {}", read_res);
        println!("Checking id");
    }

    let res = check_read_id(read_res, id);
    if res != 0 {
        abort();
    }

    done.lock().unwrap().store(true, Ordering::Relaxed);
}

fn leader_run(ptr: *mut Bundle) {
    let bundle: *mut Bundle = ptr as *mut Bundle;
    let bundle_ref = unsafe { &mut *bundle };

    let mut fp = File::create("/is_leader.tmp").expect("Unable to create file");
    fp.write_all(b"1").expect("Unable to write data");

    loop {
        if let Some(op) = bundle_ref.op {
            op();
        }
    }
}

fn empty_leader_operation() {
    println!("The good government governs little. The best government does not govern at all");
    thread::sleep(Duration::from_secs(10));
}

fn wait_timeout(done: &Arc<Mutex<AtomicBool>>, timeout: Duration) -> bool {
    let start_time = std::time::Instant::now();
    let end_time = start_time + timeout;

    while std::time::Instant::now() < end_time {
        // Check if the value of DONE has changed
        if done.lock().unwrap().load(Ordering::Relaxed) {
            break;
        }
    }

    if !done.lock().unwrap().load(Ordering::Relaxed) {
        if unsafe { VERBOSE } {
            println!("DONE has not changed. timeout!");
        }
        return true;
    }

    if unsafe { VERBOSE } {
        println!("DONE has changed. so no timeout!");
    }
    return false;
}

fn write_latency(w_start: Instant, w_end: Instant, filename: &str) {
    let latency = w_end - w_start;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)
        .expect("Failed to open file");

    writeln!(file, "{}", latency.as_millis()).expect("Failed to write to file");
}

fn leader_loop(mut b: Bundle, id: u64) {
    let mut leader: bool = true;
    if b.op == None {
        b.op = Some(empty_leader_operation);
    }

    let leader_thread = thread::spawn(move || leader_run(&mut b));
    // Create an Arc to share the Mutex wrapped AtomicBool across threads
    let leader_read_done = Arc::new(Mutex::new(AtomicBool::new(false)));

    while leader {
        let mut zero = Arc::new(Mutex::new(AtomicBool::new(false)));
        // we just wait for the term. No need to check res.
        wait_timeout(&mut zero, Duration::from_millis(LEADERTIMEOUT));
        let leader_start = Instant::now();

        // Spawn a thread to execute the read function
        let leader_reader_shared_clone = leader_read_done.clone();
        thread::spawn(move || {
            read_from_election_block_caller(&leader_reader_shared_clone, id);
        });

        if unsafe { VERBOSE } {
            println!("Remaining time: {:?}", READTIMEOUT);
        }

        let res = wait_timeout(&leader_read_done, Duration::from_millis(READTIMEOUT));
        let leader_end = Instant::now();
        write_latency(leader_start, leader_end, "/reelection_latencies.tmp");
        leader_read_done
            .lock()
            .unwrap()
            .store(false, Ordering::Relaxed);

        if res {
            println!("Timeout occured for leader! Quitting!");
            leader_thread.thread().unpark();
            leader = false;
        }
    }
}

pub(crate) fn run_election(b: Bundle) {
    let id: u64 = init();

    // Create an Arc to share the Mutex wrapped AtomicBool across threads
    let write_done = Arc::new(Mutex::new(AtomicBool::new(false)));
    // Spawn a thread to execute the write function
    let writer_shared_clone = write_done.clone();
    thread::spawn(move || {
        write_and_check_election_block(&writer_shared_clone, id);
    });

    if unsafe { VERBOSE } {
        println!("main - Waiting write");
    }

    let res = wait_timeout(&write_done, Duration::from_millis(WRITETIMEOUT));
    if res {
        println!("error - timedout");
        abort();
    }

    let zero = Arc::new(Mutex::new(AtomicBool::new(false)));
    wait_timeout(&zero, Duration::from_millis(TERM));

    if unsafe { VERBOSE } {
        println!("main - Waiting term");
    }

    // Create an Arc to share the Mutex wrapped AtomicBool across threads
    let read_done = Arc::new(Mutex::new(AtomicBool::new(false)));
    // Spawn a thread to execute the read function
    let reader_shared_clone = read_done.clone();
    thread::spawn(move || {
        read_from_election_block_caller(&reader_shared_clone, id);
    });

    let res = wait_timeout(&read_done, Duration::from_millis(READTIMEOUT));
    if res {
        println!("main - aborting after timedout, reading from election block caller");
        abort();
    }

    if unsafe { VERBOSE } {
        println!("main - Entering leader loop");
    }

    leader_loop(b, id);
}
