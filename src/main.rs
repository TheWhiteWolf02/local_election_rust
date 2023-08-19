mod emmc_client;
mod run_election;

static mut GLOBAL_VARIABLE: u64 = 42;
use crate::run_election::*;

fn main() {
    println!("Hello, world!");

    let bundle = Bundle { op: None };
    run_election(bundle);

    /*
    // for testing emmc client
    let server_addresses = ServerAddresses {
        read: SocketAddr::new(IpAddr::from([127, 0, 0, 1]), EMMCPORT),
        write: SocketAddr::new(IpAddr::from([127, 0, 0, 1]), EMMCPORTR),
    };

    init_emmc(&server_addresses);

    let read_id = {
        let read_socket = init_read_socket(&server_addresses.read)
            .expect("Unable to re-initialize read socket");
        _read_from_election_block(&read_socket)
            .expect("Error reading from election block")
    };
    println!("Read ID: {}", read_id);

    let write_socket = init_write_socket(&server_addresses.write)
        .expect("Unable to re-initialize write socket");
    _write_to_election_block(&write_socket, read_id)
        .expect("Error writing to election block");
    */
}
