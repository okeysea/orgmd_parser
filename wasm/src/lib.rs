extern crate wasm_bindgen;

use parser::ast::*;
use parser::md_parser::md_parse;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}

#[wasm_bindgen]
pub fn parse_markdown(source: &str) -> String {
    let mut node = ASTNode::new(ASTElm {
        ..Default::default()
    });
    node = md_parse(source, node);
    let serialized = serde_json::to_string(&node).unwrap();
    serialized
}
