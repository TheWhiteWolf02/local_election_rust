use std::alloc::GlobalAlloc;
use std::net::{UdpSocket, SocketAddr};
use std::env;
use std::str::from_utf8;

use crate::GLOBAL_VARIABLE;

pub const EMMCPORT: u16 = 8889;
pub const EMMCPORTR: u16 = 8888;
const MAX_UINT64_SIZE: usize = 21;
const READEB_CMD: &str = "READ_EB";
// const READEB_CMD_SIZE: usize = 8;

pub struct ServerAddresses {
    pub read: SocketAddr,
    pub write: SocketAddr,
}

pub fn init_write_socket(server_addr: &SocketAddr) -> std::io::Result<UdpSocket> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect(server_addr)?;
    Ok(socket)
}

pub fn init_read_socket(server_addr: &SocketAddr) -> std::io::Result<UdpSocket> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect(server_addr)?;
    Ok(socket)
}

pub fn init_emmc(server_addresses: &ServerAddresses) {
    let emmc_ip = env::var("EMMC_ADDRESS").expect("EMMC_ADDRESS not set");
    println!("init emmc - emmc_ip: {}", emmc_ip);

    let _read_socket = init_read_socket(&server_addresses.read)
        .expect("ERROR - Unable to initialize read socket");
    println!("init emmc - read socket initialized");

    let _write_socket = init_write_socket(&server_addresses.write)
        .expect("ERROR - Unable to initialize write socket");
    println!("init emmc - write socket initialized");
}

pub fn _read_from_election_block(socket: &UdpSocket) -> std::io::Result<u64> {
    // for local testing purposes
    let read_id: u64;
    unsafe {
        read_id = GLOBAL_VARIABLE;
    }
    Ok(read_id)
    /*
    let cmd = READEB_CMD.to_string();
    socket.send(cmd.as_bytes())?;

    let mut str_id = [0u8; MAX_UINT64_SIZE];
    socket.recv(&mut str_id)?;
    
    let str_id = from_utf8(&str_id)
        .expect("Invalid UTF-8")
        .trim();

    let read_id = str_id.parse()
        .expect("Invalid u64");
    
    Ok(read_id);
    */
}

pub fn _write_to_election_block(socket: &UdpSocket, id: u64) -> std::io::Result<u64> {
    // for local testing purposes
    unsafe {
        GLOBAL_VARIABLE = id;
        println!("write ID: {}", GLOBAL_VARIABLE);
    }
    let str_id = id.to_string();
    socket.send(str_id.as_bytes())?;
    Ok(0)
}
