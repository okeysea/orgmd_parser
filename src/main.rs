use orgmd_parser::module::ast::{
    ASTNode, ASTElm, 
};
use orgmd_parser::module::md_parser::md_parse;

fn main() {
    let mut node = ASTNode::new( ASTElm { ..Default::default() } );
    node = md_parse("\
# headering
*toside
*toside
emphasis*
emphasis*\
", node);
    println!("{:?}", node);
    println!("{:?}", node.render_debug_format());
}

