extern crate wasm_bindgen;

pub mod module;

use module::ast::*;
use module::md_parser::md_parse;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}

#[wasm_bindgen]
pub fn parse_markdown(source: &str) -> String {
    let mut node = ASTNode::new( ASTElm { ..Default::default() } );
    md_parse(source, node);
}

