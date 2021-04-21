use parser::ast::{ASTElm, ASTNode};
use parser::md_parser::md_parse;

fn main() {
    let mut node = ASTNode::new(ASTElm {
        ..Default::default()
    });
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
    node = md_parse(
        "\
# headering
#kigouwotukattemasu
# headering*kigou
# headering*emphasis* kigou*desu
paragraph(kakko)
",

        node,
    );
    println!("{:?}", node);
    println!("{:?}", node.render_debug_format());
    let content = serde_json::to_string_pretty(&node).unwrap();
    println!("{}", content);
}
