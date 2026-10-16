const TEXT: &str = r#"
clk: # 3  ...
regions:  start:2 | handshake : 3 | transfer:2+ | end:
v @1 "a" @4 "assert" @8 "ack"
ctrl: t b 101.0x.10
^ @2 "held"
v @6 "b"
data: b x.[3:00h][FFh][...]x.
note a : some some information about the signal
note b : as many transfers as needed at a rate of one transfer per clock cycle
"#;
fn main() {
    print!("{}", splot::render(TEXT));
}
