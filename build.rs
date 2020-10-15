/**
stdout of build.rs: target/debug/build/pg_client-d3cb3368e7e2ca08/output
stderr of build.rs: target/debug/build/pg_client-d3cb3368e7e2ca08/stderr
可能用得上的环境变量:
CARGO_CFG_TARGET_ARCH: x86_64
CARGO_CFG_TARGET_OS: macos
CARGO_CFG_TARGET_ENDIAN: little

值得关注的编译时环境变量:
CARGO_CFG_TARGET_POINTER_WIDTH: 64
CARGO_CFG_TARGET_HAS_ATOMIC*
*/
fn main() {
    for (key, value) in std::env::vars() {
        println!("{}: {}", key, value);
    }
}