use std::io::{Read, Write};

// 196608
const PG_PROTOCOL_VERSION_3: i32 = 0x00_03_00_00;
const PG_DEFAULT_PORT: u16 = 5432;
const AUTHENTICATION_TAG: u8 = b'R';
const PARAMETER_STATUS_TAG: u8 = b'S';
const AUTHENTICATION_OK: i32 = 0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // StartupMessage { user: postgres, database: postgres }
    let mut start_up_msg_body: Vec<u8> = Vec::new();
    start_up_msg_body.extend(&PG_PROTOCOL_VERSION_3.to_be_bytes());
    // pg通信中字符串实际上是std::ffi::CStr类型
    start_up_msg_body.extend(b"user");
    start_up_msg_body.push(0u8);
    start_up_msg_body.extend(b"postgres");
    start_up_msg_body.push(0u8);
    // terminator of start_up_msg_body
    start_up_msg_body.push(0u8);
    let body_len = start_up_msg_body.len() as u32 + 4u32;

    let mut start_up_msg: Vec<u8> = Vec::new();
    start_up_msg.extend(&body_len.to_be_bytes());
    start_up_msg.append(&mut start_up_msg_body);

    let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{}", PG_DEFAULT_PORT))?;
    stream.write(start_up_msg.as_slice())?;

    // let mut cursor = std::io::Cursor::new(vec![0u8; 512]);
    // stream.read(&mut cursor.get_mut())?;
    // cursor.read(&mut [0u8; 1]);
    let mut read_buff = [0u8; 512];
    stream.read(&mut read_buff)?;
    println!("{:?}", read_buff);

    // AUTHENTICATION resp
    assert_eq!(read_buff[0], AUTHENTICATION_TAG);
    let resp_len = read_4_bytes_slice_to_i32(&read_buff[1..=4]);
    assert_eq!(resp_len, 8);
    let auth_res = read_4_bytes_slice_to_i32(&read_buff[5..=8]);
    assert_eq!(auth_res, AUTHENTICATION_OK);

    // PARAMETER_STATUS resp
    assert_eq!(read_buff[9], PARAMETER_STATUS_TAG);
    let resp_len = read_4_bytes_slice_to_i32(&read_buff[10..=13]);
    dbg!(String::from_utf8(read_buff[14..14+resp_len as usize].to_vec()));

    stream.shutdown(std::net::Shutdown::Both)?;
    Ok(())
}

fn read_4_bytes_slice_to_i32(bytes: &[u8]) -> i32 {
    let mut buff_len = [0u8; 4];
    buff_len.copy_from_slice(bytes);
    i32::from_be_bytes(buff_len)
}


// [82, 0, 0, 0, 8, 0, 0, 0, 0,
// 83, 0, 0, 0, 22, 97, 112, 112, 108, 105, 99, 97, 116, 105, 111, 110, 95, 110, 97, 109, 101, 0, 0,
// 83, 0, 0, 0, 25, 99, 108, 105, 101, 110, 116, 95, 101, 110, 99, 111, 100, 105, 110, 103, 0, 85, 84, 70, 56, 0,
// 83, 0, 0, 0, 23, 68, 97, 116, 101, 83, 116, 121, 108, 101, 0, 73, 83, 79, 44, 32, 77, 68, 89, 0, 83, 0, 0, 0, 25, 105, 110, 116, 101, 103, 101, 114, 95, 100, 97, 116, 101, 116, 105, 109, 101, 115, 0, 111, 110, 0, 83, 0, 0, 0, 27, 73, 110, 116, 101, 114, 118, 97, 108, 83, 116, 121, 108, 101, 0, 112, 111, 115, 116, 103, 114, 101, 115, 0, 83, 0, 0, 0, 20, 105, 115, 95, 115, 117, 112, 101, 114, 117, 115, 101, 114, 0, 111, 110, 0, 83, 0, 0, 0, 25, 115, 101, 114, 118, 101, 114, 95, 101, 110, 99, 111, 100, 105, 110, 103, 0, 85, 84, 70, 56, 0, 83, 0, 0, 0, 24, 115, 101, 114, 118, 101, 114, 95, 118, 101, 114, 115, 105, 111, 110, 0, 49, 50, 46, 52, 0, 83, 0, 0, 0, 35, 115, 101, 115, 115, 105, 111, 110, 95, 97, 117, 116, 104, 111, 114, 105, 122, 97, 116, 105, 111, 110, 0, 112, 111, 115, 116, 103, 114, 101, 115, 0, 83, 0, 0, 0, 35, 115, 116, 97, 110, 100, 97, 114, 100, 95, 99, 111, 110, 102, 111, 114, 109, 105, 110, 103, 95, 115, 116, 114, 105, 110, 103, 115, 0, 111, 110, 0, 83, 0, 0, 0, 27, 84, 105, 109, 101, 90, 111, 110, 101, 0, 65, 115, 105, 97, 47, 83, 104, 97, 110, 103, 104, 97, 105, 0, 75, 0, 0, 0, 12, 0, 1, 82, 152, 171, 183, 94, 73, 90, 0, 0, 0, 5, 73, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
#[test]
fn test() {
    dbg!(String::from_utf8(vec![97, 112, 112, 108, 105, 99, 97, 116, 105, 111, 110, 95, 110, 97, 109, 101]).unwrap());
    dbg!(String::from_utf8(vec![99, 108, 105, 101, 110, 116, 95, 101, 110, 99, 111, 100, 105, 110, 103]).unwrap());
    dbg!(String::from_utf8(vec![85, 84, 70, 56]).unwrap());
}
