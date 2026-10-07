const TEXT: &str = r#"
    clk: 2 ...
    regions:  start:3 | middle_name_too_long : 2 | end:4+ | _:
    note a : some text
    note b : some longer text that should exceed the maximum width allocated through cycles
"#;
fn main() {
    print!("{}", splot::render(TEXT));
}
