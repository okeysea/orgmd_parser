pub mod module;
use module::ast::{
    ASTNode, ASTElm, 
};
use module::md_parser::md_parse;

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
# これは強調のテストです
*fake
*入れ子のテストです
*入れ子2
*一行で完結する行*
入れ子2の終了*
入れ子の終了*
# ここで強調はおわっています
", node);
    println!("{:?}", node);
    println!("{:?}", node.render_debug_format());
    let content = serde_json::to_string_pretty(&node).unwrap();
    println!("{}", content);
}
