use wasm_bindgen::prelude::wasm_bindgen;

mod model;
mod parser;
mod renderer;

#[wasm_bindgen]
pub fn render(input: &str) -> String {
    let model = model::Model::parse(input);
    renderer::Canvas::render(&model).to_string()
}
