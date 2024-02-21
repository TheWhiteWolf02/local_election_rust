mod emmc_client;
mod run_election;
mod server;

use std::thread;
use std::time::Duration;

use crate::run_election::*;
use crate::server::*;

fn main() {
    /* 
    println!("Hello, world!");
    let server_thread = std::thread::spawn(|| {
        if let Err(err) = start_server() {
            eprintln!("Server error: {}", err);
        }
    });

    thread::sleep(Duration::from_secs(5));
    */
    let bundle = Bundle { op: None };
    run_election(bundle);

    //server_thread.join().expect("Server thread panicked");
}
