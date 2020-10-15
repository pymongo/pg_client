#![feature(cstring_from_vec_with_nul)]

use std::io::{BufRead, Write};

// 196608
const PG_PROTOCOL_VERSION_3: i32 = 0x00_03_00_00;
const PG_DEFAULT_PORT: u16 = 5432;
#[allow(dead_code)]
const AUTHENTICATION_OK: i32 = 0i32;

#[allow(dead_code)]
#[repr(u8)]
enum MessageType {
    BackendKeyData = b'K',
    SimpleQuery = b'Q',
    Authentication = b'R',
    ParameterStatus = b'S',
    ReadyForQuery = b'Z'
}

#[allow(dead_code)]
#[repr(u8)]
enum PgSessionStatus {
    Idle = b'I',
    DoingTransaction = b'T',
    ErrorInTransaction = b'E'
}

struct PgRespParser {
    cursor: usize,
    data: Vec<u8>,
}

impl PgRespParser {
    fn new(data: Vec<u8>) -> Self {
        PgRespParser { cursor: 0, data }
    }

    fn read_a_u8(&mut self) -> u8 {
        let res = self.data[self.cursor];
        self.cursor += 1;
        res
    }

    fn read_a_i32(&mut self) -> i32 {
        let mut buff = [0u8; 4];
        buff.copy_from_slice(&self.data[self.cursor..self.cursor + 4]);
        self.cursor += 4;
        i32::from_be_bytes(buff)
    }

    fn read_a_cstr(&mut self) -> std::ffi::CString {
        for nul_terminator in self.cursor..self.data.len() {
            if self.data[nul_terminator] == 0 {
                let cstr = std::ffi::CString::from_vec_with_nul(
                    self.data[self.cursor..=nul_terminator].to_vec(),
                )
                .unwrap();
                self.cursor = nul_terminator + 1;
                return cstr;
            }
        }
        unreachable!()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // StartupMessage { user: postgres, database: postgres }
    let mut startup_msg_body: Vec<u8> = Vec::new();
    startup_msg_body.extend(&PG_PROTOCOL_VERSION_3.to_be_bytes());
    // pg通信中字符串实际上是std::ffi::CStr类型
    startup_msg_body.extend(b"user");
    // with nul terminator
    startup_msg_body.push(0u8);
    startup_msg_body
        .extend(std::ffi::CStr::from_bytes_with_nul(b"postgres\0")?.to_bytes_with_nul());
    // terminator of start_up_msg_body
    startup_msg_body.push(0u8);
    let body_len = startup_msg_body.len() as u32 + 4u32;

    let mut startup_msg: Vec<u8> = Vec::new();
    startup_msg.extend(&body_len.to_be_bytes());
    startup_msg.append(&mut startup_msg_body);

    // let mut cursor = std::io::Cursor::new(vec![0u8; 10]);
    // cursor.fill_buf();
    let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{}", PG_DEFAULT_PORT))?;
    let mut reader = std::io::BufReader::new(&mut stream);
    reader.get_mut().write(startup_msg.as_slice())?;
    let mut startup_resp = PgRespParser::new(reader.fill_buf()?.to_vec());

    // AUTH resp
    assert_eq!(startup_resp.read_a_u8(), MessageType::Authentication as u8);
    let resp_len = startup_resp.read_a_i32();
    assert_eq!(resp_len, 8);
    let auth_res = startup_resp.read_a_i32();
    assert_eq!(auth_res, AUTHENTICATION_OK);

    // PARAMETER_STATUS resp
    while startup_resp.read_a_u8() == MessageType::ParameterStatus as u8 {
        let _resp_len = startup_resp.read_a_i32();
        let key = startup_resp.read_a_cstr();
        let value = startup_resp.read_a_cstr();
        println!("{:?}: {:?}", key, value);
    }

    // BackendKeyData resp
    let resp_len = startup_resp.read_a_i32();
    assert_eq!(resp_len, 12);
    let pg_server_process_id = startup_resp.read_a_i32();
    let secret_key = startup_resp.read_a_i32();
    dbg!(pg_server_process_id, secret_key);

    // ReadyForQuery resp
    assert_eq!(startup_resp.read_a_u8(), MessageType::ReadyForQuery as u8);
    let resp_len = startup_resp.read_a_i32();
    assert_eq!(resp_len, 5);
    assert_eq!(startup_resp.read_a_u8(), PgSessionStatus::Idle as u8);

    // https://thepacketgeek.com/rust/tcpstream/reading-and-writing/
    // Mark the bytes read as consumed so the buffer will not return them in a subsequent/next read
    // fill_buf一定要搭配consume使用，读完本次TcpStream的数据后，将cursor前移，标记这块已读过的数组区域为空闲空间，下次从stream中读取数据时可以覆盖掉这片区域
    // 消息不完整的话，要将已读放回缓存区区
    reader.consume(startup_resp.data.len());

    let mut query_body: Vec<u8> = Vec::new();
    // pg通信中字符串实际上是std::ffi::CStr类型
    query_body.extend(b"SELECT 1::char;");
    query_body.push(0u8);

    let mut query_msg: Vec<u8> = vec![MessageType::SimpleQuery as u8];
    let body_len = query_body.len() as u32 + 4u32;
    query_msg.extend(&body_len.to_be_bytes());
    query_msg.append(&mut query_body);
    reader.get_mut().write(query_msg.as_slice())?;
    let resp = reader.fill_buf()?.to_vec();
    println!("{:?}", resp);
    Ok(())
}


/* Response of StartupMessage
82, 0, 0, 0, 8, 0, 0, 0, 0,

83, 0, 0, 0, 22, 97, 112, 112, 108, 105, 99, 97, 116, 105, 111, 110, 95, 110, 97, 109, 101, 0, 0,
83, 0, 0, 0, 25, 99, 108, 105, 101, 110, 116, 95, 101, 110, 99, 111, 100, 105, 110, 103, 0, 85, 84, 70, 56, 0,
83, 0, 0, 0, 23, 68, 97, 116, 101, 83, 116, 121, 108, 101, 0, 73, 83, 79, 44, 32, 77, 68, 89, 0,
83, 0, 0, 0, 25, 105, 110, 116, 101, 103, 101, 114, 95, 100, 97, 116, 101, 116, 105, 109, 101, 115, 0, 111, 110, 0,
83, 0, 0, 0, 27, 73, 110, 116, 101, 114, 118, 97, 108, 83, 116, 121, 108, 101, 0, 112, 111, 115, 116, 103, 114, 101, 115, 0,
83, 0, 0, 0, 20, 105, 115, 95, 115, 117, 112, 101, 114, 117, 115, 101, 114, 0, 111, 110, 0,
83, 0, 0, 0, 25, 115, 101, 114, 118, 101, 114, 95, 101, 110, 99, 111, 100, 105, 110, 103, 0, 85, 84, 70, 56, 0,
83, 0, 0, 0, 24, 115, 101, 114, 118, 101, 114, 95, 118, 101, 114, 115, 105, 111, 110, 0, 49, 50, 46, 52, 0,
83, 0, 0, 0, 35, 115, 101, 115, 115, 105, 111, 110, 95, 97, 117, 116, 104, 111, 114, 105, 122, 97, 116, 105, 111, 110, 0, 112, 111, 115, 116, 103, 114, 101, 115, 0,
83, 0, 0, 0, 35, 115, 116, 97, 110, 100, 97, 114, 100, 95, 99, 111, 110, 102, 111, 114, 109, 105, 110, 103, 95, 115, 116, 114, 105, 110, 103, 115, 0, 111, 110, 0,
83, 0, 0, 0, 27, 84, 105, 109, 101, 90, 111, 110, 101, 0, 65, 115, 105, 97, 47, 83, 104, 97, 110, 103, 104, 97, 105, 0,

75, 0, 0, 0, 12, 0, 1, 82, 152, 171, 183, 94, 73,
90, 0, 0, 0, 5, 73

params response:
"application_name": ""
"client_encoding": "UTF8"
"DateStyle": "ISO, MDY"
"integer_datetimes": "on"
"IntervalStyle": "postgres"
"is_superuser": "on"
"server_encoding": "UTF8"
"server_version": "12.4"
"session_authorization": "postgres"
"standard_conforming_strings": "on"
"TimeZone": "Asia/Shanghai"
*/
#[test]
fn test() {
    dbg!(String::from_utf8(vec![
        99, 108, 105, 101, 110, 116, 95, 101, 110, 99, 111, 100, 105, 110, 103
    ])
    .unwrap());
    dbg!(String::from_utf8(vec![85, 84, 70, 56]).unwrap());
}
