use std::fmt::Formatter;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum MessageType {
    /// Server发的'C'表示CommandComplete，Client发的'C'表示Close
    CloseOrCommandComplete = b'C', // 67
    DataRowOrDescribe = b'D', // 68
    BackendKeyData = b'K',
    SimpleQuery = b'Q',
    Authentication = b'R',
    ParameterStatus = b'S',
    /// client SimpleQuery request success, server response a RowDescription first
    RowDescription = b'T', // 84
    ReadyForQuery = b'Z' // 90
}

impl From<u8> for MessageType {
    fn from(message_type: u8) -> Self {
        match message_type {
            b'C' => Self::CloseOrCommandComplete,
            b'D' => Self::DataRowOrDescribe,
            b'K' => Self::BackendKeyData,
            b'Q' => Self::SimpleQuery,
            b'R' => Self::Authentication,
            b'S' => Self::ParameterStatus,
            b'T' => Self::RowDescription,
            b'Z' => Self::ReadyForQuery,
            _ => panic!("Unknown message type {}", message_type),
        }
    }
}

/// not include StartupMessage, ignore message_len part
pub struct Message {
    pub msg_type: MessageType,
    pub body: Vec<u8>
}

impl Message {
    pub fn new(msg_type: MessageType, body: Vec<u8>) -> Self {
        Self {
            msg_type,
            body
        }
    }

    pub fn to_vec_u8(&self) -> Vec<u8> {
        let body_len = self.body.len();
        let mut bytes = Vec::with_capacity(1+4+body_len);
        bytes.push(self.msg_type as u8);
        // len of bytes exclude message_type(first byte)
        bytes.extend(&(body_len as i32 + 4).to_be_bytes());
        bytes.extend_from_slice(&self.body);
        bytes
    }
}

/// 不喜欢Debug竖着打印数组，那我自己实现一个打印方法
impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{\n\ttype: {:?}({}),\n\tbody: {:?} \n}}", self.msg_type, self.msg_type as u8, self.body)
    }
}
