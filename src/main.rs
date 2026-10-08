const TEXT: &str = r#"
    clk: 2 ...
    regions:  start:3 | transfer : 2 | end:1+ | _:
    ctrl: 0..1.0..
    data: x..[FFh][AAh][00h]x.
    note a : some text
    note b : some longer text that should exceed the maximum width allocated through cycles
"#;
fn main() {
    print!("{}", splot::render(TEXT));
}
