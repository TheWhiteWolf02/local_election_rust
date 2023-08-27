use std::net::UdpSocket;

const EMMCPORT: u16 = 24000;
const MAX_UINT64_SIZE: usize = 21;
const READEB_CMD: &str = "READ_EB";

pub fn start_server() -> std::io::Result<()> {
    let server_addr = format!("127.0.0.1:{}", EMMCPORT);
    let socket = UdpSocket::bind(&server_addr)?;

    // later read from memory
    let mut election_block_value: u64 = 0;

    loop {
        let mut buf = [0u8; MAX_UINT64_SIZE];
        let (_, client_addr) = socket.recv_from(&mut buf)?;

        let received_cmd = std::str::from_utf8(&buf)
                    .expect("Invalid UTF-8")
                    .trim()
                    .trim_end_matches('\0');

        println!("[SERVER] received_cmd after parsing {}", received_cmd);

        if received_cmd.contains(READEB_CMD) {
            println!(
                "[SERVER] Reading from election block! {}",
                election_block_value
            );

            socket.send_to(election_block_value.to_string().as_bytes(), client_addr)?;
        } else {
            let new_value: u64 = received_cmd.parse()
                    .expect("parse failed to u64");

            election_block_value = new_value;
            println!(
                "[SERVER] Writing to election block! {}",
                election_block_value
            );
        }
    }
}
