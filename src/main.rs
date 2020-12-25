use orgmd_parser::module::ast::{
    ASTNode, ASTElm, 
};
use orgmd_parser::module::md_parser::md_parse;

fn main() {
    let mut node = ASTNode::new( ASTElm { ..Default::default() } );
/*
    node = md_parse("\
# headering
*emphasis*
*wideline
emphasis*

*nested
*emphasis
level2*
level*
", node);
*/
    node = md_parse("\
*入れ子のテストです
*入れ子2
*一行で完結する行*
入れ子2の終了*
入れ子の終了*
", node);
    println!("{:?}", node);
    println!("{:?}", node.render_debug_format());
}

