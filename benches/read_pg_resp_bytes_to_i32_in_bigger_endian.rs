/*!
解析pg返回的Message中2-5的4个字节组成的i32(表示消息的长度)的两个方法:

*/
#![feature(test)]
extern crate test;

const PG_STARTUP_MSG_AUTHENTICATION_OK_RESP: [u8; 9] = [82, 0, 0, 0, 8, 0, 0, 0, 0];

/**
Like byteorder create read_u32 method
*/
#[bench]
fn copy_to_a_new_4_bytes_array_and_convert(bencher: &mut test::Bencher) {
    bencher.iter(|| {
        let mut buff_msg_len = [0u8; 4];
        buff_msg_len.copy_from_slice(&PG_STARTUP_MSG_AUTHENTICATION_OK_RESP[1..5]);
        let resp_msg_len = i32::from_be_bytes(buff_msg_len);
        assert_eq!(resp_msg_len, 8);
    });
}

#[bench]
fn calc_len_by_byte_shift(bencher: &mut test::Bencher) {
    bencher.iter(|| {
        let mut resp_msg_len = 0i32;
        resp_msg_len += (PG_STARTUP_MSG_AUTHENTICATION_OK_RESP[1] as i32) << (3 * 8);
        resp_msg_len += (PG_STARTUP_MSG_AUTHENTICATION_OK_RESP[2] as i32) << (2 * 8);
        resp_msg_len += (PG_STARTUP_MSG_AUTHENTICATION_OK_RESP[3] as i32) << (1 * 8);
        resp_msg_len += (PG_STARTUP_MSG_AUTHENTICATION_OK_RESP[4] as i32);
        assert_eq!(resp_msg_len, 8);
    });
}

