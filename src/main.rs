use std::io::{Read, Write};

const PG_PROTOCOL_VERSION_3: i32 = 0x00_03_00_00;
const PG_DEFAULT_PORT: u16 = 5432;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // user: postgres, database: postgres
    let mut start_up_msg_body: Vec<u8> = Vec::new();
    start_up_msg_body.extend(&PG_PROTOCOL_VERSION_3.to_be_bytes());
    start_up_msg_body.extend("user".bytes());
    start_up_msg_body.push(0u8);
    start_up_msg_body.extend(b"postgres");
    start_up_msg_body.push(0u8);
    // terminator of start_up_msg_body
    start_up_msg_body.push(0u8);
    let body_len = start_up_msg_body.len() as u32 + 4u32;

    let mut start_up_msg: Vec<u8> = Vec::new();
    start_up_msg.extend(&body_len.to_be_bytes());
    start_up_msg.extend(start_up_msg_body);

    let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{}", PG_DEFAULT_PORT))?;
    stream.write(start_up_msg.as_slice())?;
    let mut read_buffer: [u8; 128] = [0; 128];
    stream.read(&mut read_buffer)?;
    println!("{:?}", read_buffer);
    Ok(())
}
