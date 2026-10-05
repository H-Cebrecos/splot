mod model;
mod parser;
mod renderer;

fn main() {
    let clock = parser::parse_clk("clk:analog falling 1 ...");

    println!("{clock:#?}");
}
