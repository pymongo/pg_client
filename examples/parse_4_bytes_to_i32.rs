fn main() {
    use std::convert::TryInto;

    fn read_be_i32(input: &mut &[u8]) -> i32 {
        let (int_bytes, rest) = input.split_at(std::mem::size_of::<i32>());
        *input = rest;
        i32::from_be_bytes(int_bytes.try_into().unwrap())
    }

    let mut v: Vec<u8> = Vec::with_capacity(4);

    v.push(1);
    v.push(2);
    v.push(3);
    v.push(4);

    let _i = read_be_i32(&mut v.as_slice());
}
