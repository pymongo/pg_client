/*!
## Notes:

1. ParameterStatusMessage, StartupMessage里所有字符串key-value pair的内存布局都是两个CStr挨着组成key-value

例如: 83, 0, 0, 0, 25, 99, 108, 105, 101, 110, 116, 95, 101, 110, 99, 111, 100, 105, 110, 103, 0, 85, 84, 70, 56, 0

表示: key-value pair "client_encoding": "UTF8"

2. Int32指的是i32还是u32?

## Message Example

### client request StartupMessage

```
let mut startup_msg_body: Vec<u8> = Vec::new();
startup_msg_body.extend(&0x00_03_00_00.to_be_bytes());
startup_msg_body.extend(b"user\0");
startup_msg_body.extend(b"postgres\0");
// terminator of startup_msg_body, only startup_message has terminator and without first byte message type(historical reason)
startup_msg_body.push(0u8);
let body_len = startup_msg_body.len() as u32 + 4u32;
let mut startup_msg: Vec<u8> = Vec::new();
startup_msg.extend(&body_len.to_be_bytes());
startup_msg.append(&mut startup_msg_body);
```

### server response StartupMessage

```text
82, 0, 0, 0, 8, 0, 0, 0, 0 ReadyForQuery { pg_session_status: i32 }

Multi ParameterStatus { key: CStr, value: CStr }
83, 0, 0, 0, 22, 97, 112, 112, 108, 105, 99, 97, 116, 105, 111, 110, 95, 110, 97, 109, 101, 0, 0
83, 0, 0, 0, 25, 99, 108, 105, 101, 110, 116, 95, 101, 110, 99, 111, 100, 105, 110, 103, 0, 85, 84, 70, 56, 0
83, 0, 0, 0, 23, 68, 97, 116, 101, 83, 116, 121, 108, 101, 0, 73, 83, 79, 44, 32, 77, 68, 89, 0
83, 0, 0, 0, 25, 105, 110, 116, 101, 103, 101, 114, 95, 100, 97, 116, 101, 116, 105, 109, 101, 115, 0, 111, 110, 0
83, 0, 0, 0, 27, 73, 110, 116, 101, 114, 118, 97, 108, 83, 116, 121, 108, 101, 0, 112, 111, 115, 116, 103, 114, 101, 115, 0
83, 0, 0, 0, 20, 105, 115, 95, 115, 117, 112, 101, 114, 117, 115, 101, 114, 0, 111, 110, 0
83, 0, 0, 0, 25, 115, 101, 114, 118, 101, 114, 95, 101, 110, 99, 111, 100, 105, 110, 103, 0, 85, 84, 70, 56, 0
83, 0, 0, 0, 24, 115, 101, 114, 118, 101, 114, 95, 118, 101, 114, 115, 105, 111, 110, 0, 49, 50, 46, 52, 0
83, 0, 0, 0, 35, 115, 101, 115, 115, 105, 111, 110, 95, 97, 117, 116, 104, 111, 114, 105, 122, 97, 116, 105, 111, 110, 0, 112, 111, 115, 116, 103, 114, 101, 115, 0
83, 0, 0, 0, 35, 115, 116, 97, 110, 100, 97, 114, 100, 95, 99, 111, 110, 102, 111, 114, 109, 105, 110, 103, 95, 115, 116, 114, 105, 110, 103, 115, 0, 111, 110, 0
83, 0, 0, 0, 27, 84, 105, 109, 101, 90, 111, 110, 101, 0, 65, 115, 105, 97, 47, 83, 104, 97, 110, 103, 104, 97, 105, 0
Multi ParameterStatus {
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
}

75, 0, 0, 0, 12, 0, 1, 82, 152, 171, 183, 94, 73 BackendKeyData { process_id: i32, secret_key: i32 }
90, 0, 0, 0, 5, 73 ReadyForQuery { pg_session_status: u8 }
```
*/
#![allow(dead_code)]
mod message;
use message::{Message, MessageType};

use std::io::{BufRead, Write};
use std::convert::TryInto;

const PG_DEFAULT_PORT: u16 = 5432;
const PG_PROTOCOL_VERSION_3: i32 = 0x00_03_00_00; // 196608
const AUTHENTICATION_OK: i32 = 0i32;

#[repr(u8)]
enum PgSessionStatus {
    Idle = b'I', // 73
    DoingTransaction = b'T',
    ErrorInTransaction = b'E'
}

struct PgRespParser {
    cursor: usize,
    data: Vec<u8>,
}

/// TODO check cursor index out of range
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
        res
    }

    fn read_a_cstr(&mut self) -> String {
        for nul_terminator in self.cursor..self.data.len() {
            if self.data[nul_terminator] == 0 {
                let str = String::from_utf8(self.data[self.cursor..nul_terminator].to_owned()).unwrap();
                self.cursor = nul_terminator + 1;
                return str;
            }
        }
        unreachable!()
    }

    /// assert self.cursor's next byte is a new message
    fn read_a_message(&mut self) -> Message {
        let msg_type  = MessageType::from(self.read_a_u8());
        let body_len = (self.read_a_i32()-4) as usize;
        let body = self.data[self.cursor..self.cursor+body_len].to_vec();
        self.cursor += body_len;
        Message::new(msg_type, body)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // StartupMessage { user: postgres, database: postgres }
    let mut startup_msg_body: Vec<u8> = Vec::new();
    startup_msg_body.extend(&PG_PROTOCOL_VERSION_3.to_be_bytes());
    // pg通信中字符串实际上是std::ffi::CStr类型，需要结尾有\0作为nul terminator
    startup_msg_body.extend(b"user\0");
    startup_msg_body.extend(b"postgres\0");
    // terminator of startup_msg_body, only startup_message has terminator and without first byte message type(historical reason)
    startup_msg_body.push(0u8);
    let body_len = startup_msg_body.len() as i32 + 4;

    let mut startup_msg: Vec<u8> = Vec::new();
    startup_msg.extend(&body_len.to_be_bytes());
    startup_msg.append(&mut startup_msg_body);

    let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{}", PG_DEFAULT_PORT))?;
    // Use epoll(Linux)/kqueue(BSD,mac,IOS)/IOCP(windows) async/non-blocking IO
    // a thread that continually checks whether socket is readable, calling wake() when appropriate. However, this would be quite inefficient
    stream.set_nonblocking(true).unwrap();
    /* TODO
    1. 怎么知道server发送的消息已经发完了?
    2. 怎么获取kernel/网卡上TCP的buffer?
    3. 为什么TcpStream里面的flush方法是空白的?
    4. BufReader默认缓冲区是8k，如果超过8k会怎样?
    5. 如果从缓冲区中读取到消息不完整的话，会将已读数据重新放回缓存区吗?
    */
    // let mut cursor = std::io::Cursor::new(vec![0u8; 10]);
    let mut reader = std::io::BufReader::new(&mut stream);
    // Bad: reader.get_mut().write(startup_msg.as_slice())?; // written amount is not handled. Use `Write::write_all` instead
    reader.get_mut().write_all(startup_msg.as_slice())?;
    drop(startup_msg);
    let read_data: Vec<u8>;
    // TODO wrap epoll socket data to a Future
    loop {
        match reader.fill_buf() {
            Ok(data) => {
                read_data = data.to_vec();
                break;
            },
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // println!("{:?}", e);
            }
            Err(e) => panic!("Unexpected Error: {:?}", e)
        }
    }
    let mut startup_resp = PgRespParser::new(read_data);

    let auth_msg = startup_resp.read_a_message();
    assert_eq!(auth_msg.msg_type, MessageType::Authentication);
    assert_eq!(auth_msg.body.len(), 4);
    assert_eq!(auth_msg.body.iter().sum::<u8>() as i32, AUTHENTICATION_OK);

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
    // secret_key = -1619159205为什么是负数，pg文档的Int32是i32还是u32
    let secret_key = startup_resp.read_a_i32();
    dbg!(pg_server_process_id, secret_key);

    let ready_for_query_msg = startup_resp.read_a_message();
    assert_eq!(ready_for_query_msg.msg_type, MessageType::ReadyForQuery);
    assert_eq!(ready_for_query_msg.body.len(), 1);
    assert_eq!(ready_for_query_msg.body[0], PgSessionStatus::Idle as u8);

    // https://thepacketgeek.com/rust/tcpstream/reading-and-writing/
    // Mark as consumed so the buffer will not return them in next read and allow next tcp data cover this data
    // fill_buf一定要搭配consume使用，读完本次TcpStream的数据后，将cursor前移，标记这块已读过的数组区域为空闲空间，下次从stream中读取数据时可以覆盖掉这片区域
    reader.consume(startup_resp.data.len());
    drop(startup_resp);

    let query_msg = Message::new(MessageType::SimpleQuery, b"SELECT 1::char;\0".to_vec());
    reader.get_mut().write_all(&query_msg.to_vec_u8())?;
    drop(query_msg);
    // let mut query_resp = PgRespParser::new(reader.fill_buf()?.to_vec());
    let read_data: Vec<u8>;
    // TODO wrap epoll socket data to a Future
    loop {
        match reader.fill_buf() {
            Ok(data) => {
                read_data = data.to_vec();
                break;
            },
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                println!("{:?}", e);
            }
            Err(e) => panic!("Unexpected Error: {:?}", e)
        }
    }
    let mut query_resp = PgRespParser::new(read_data);
    let row_description_msg = query_resp.read_a_message();
    dbg!(row_description_msg);

    /*
    body: 0, 1, 0, 0, 0, 1, 49
    i16(0,1): the number of column
    i32(0,0,0,1): the length of the column value
    [u8](49): the value of column, 49 is `1` in ASCII
    */
    let data_row_msg = query_resp.read_a_message();
    dbg!(data_row_msg);


    // CStr(83,69,76,69,67,84,32,49,0): "SELECT 1\0"
    let command_complete_msg = query_resp.read_a_message();
    dbg!(command_complete_msg);

    let ready_for_query_msg = query_resp.read_a_message();
    dbg!(&ready_for_query_msg);
    assert_eq!(ready_for_query_msg.msg_type, MessageType::ReadyForQuery);
    assert_eq!(ready_for_query_msg.body.len(), 1);
    assert_eq!(ready_for_query_msg.body[0], PgSessionStatus::Idle as u8);
    Ok(())
}
