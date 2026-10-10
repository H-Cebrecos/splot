const TEXT: &str = r#"
    clk: 2 ...
    regions:  start:2 | handshake : 3 | transfer:1 | end:
    ctrl: t b 101.0x10
    data: b x.[3:00h][FFh]x.
    note a : some some information about the signal
    note b : as many transfers as needed at a rate of one trasfer per clock cycle
"#;
fn main() {
    print!("{}", splot::render(TEXT));
}
