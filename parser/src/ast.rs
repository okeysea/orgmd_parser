use serde::Serialize;
use std::cell::{RefCell};
use std::cell::RefMut;
use std::rc::Rc;

type Link = Rc<RefCell<ASTNode>>;
type Position = RefCell<u32>;

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ASTPos {
    line: Position,
    ch: Position,
    pos: Position,
}

impl ASTPos {
    pub fn new(line: u32, ch: u32, pos: u32) -> ASTPos {
        ASTPos{
            line:   RefCell::new(line),
            ch:     RefCell::new(ch),
            pos:    RefCell::new(pos)
        }
    }

    pub fn increase_pos_n(&self, n: u32) {
        // NOTE: 実行時Borrowチェッカーに怒られるので、変数に一旦入れる
        let a = self.pos();
        self.set_pos( a + n );
    }
    
    pub fn increase_line_n(&self, n: u32) {
        let a = self.line();
        self.set_line( a + n );
        self.set_ch(1);
        self.increase_pos_n(n);
    }

    pub fn increase_ch_n(&self, n: u32){
        let a = self.ch();
        self.set_ch( a + n );
        self.increase_pos_n(n);
    }

    // pos_mut() のようなRust流のインターフェースにしたいが、無理っぽい？
    pub fn set_pos(&self, n: u32){
        *self.pos.borrow_mut() = n;
    }

    pub fn set_line(&self, n: u32){
        *self.line.borrow_mut() = n;
    }

    pub fn set_ch(&self, n: u32){
        *self.ch.borrow_mut() = n;
    }

    pub fn pos(&self) -> u32 {
        *self.pos.borrow()
    }

    pub fn line(&self) -> u32 {
        *self.line.borrow()
    }

    pub fn ch(&self) -> u32 {
        *self.ch.borrow()
    }

}

#[test]
fn test_ASTPos_increase_positions() {

    let pos = ASTPos::new(1,1,0);
    assert_eq!( pos.pos(), 0 );
    assert_eq!( pos.line(), 1 );
    assert_eq!( pos.ch(), 1 );

    pos.increase_pos_n(1);
    pos.increase_line_n(1);
    pos.increase_ch_n(1);

    // Pos は 文字数なので、行を進めるとき(=\nを読んだ)とchを進めるとき(=一文字読んだ)に加算されるので3
    assert_eq!( pos.pos(), 3 );
    assert_eq!( pos.line(), 2 );
    assert_eq!( pos.ch(), 2 );

}

#[derive(Debug, Default, PartialEq, Serialize)]
pub struct ASTRange {
    pub begin: ASTPos,
    pub end: ASTPos,
}

impl ASTRange {
    pub fn new(begin: ASTPos, end: ASTPos) -> Self {
        ASTRange{ begin, end }
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct ASTNode {
    #[serde(flatten)]
    data: ASTElm,

    children: Vec<Link>,
}

#[derive(Debug, Default, PartialEq, Serialize)]
pub struct ASTElm {
    pub elm_type: ASTType,
    pub elm_meta: ASTMetaData,
    pub value: String,
    pub raw_value: String, // パース前のデータ
    pub range: ASTRange,
}

impl ASTElm {

    fn build(elm_type: ASTType, elm_meta: ASTMetaData, value: &str, raw_value: &str, range: ASTRange ) -> Self {
        ASTElm { elm_type, elm_meta, value: value.to_string(), raw_value: raw_value.to_string(), range, }
    }

    pub fn new_document() -> Self {
        ASTElm {
            ..Default::default()
        }
    }

    pub fn new_paragraph(value: &str, raw_value: &str, range: ASTRange ) -> Self {
        ASTElm::build( ASTType::Paragraph, ASTMetaData::Nil, value, raw_value, range )
    }

    pub fn new_headers(level: ASTMetaData, value: &str, raw_value: &str, range: ASTRange ) -> Self {
        ASTElm::build( ASTType::Headers, level, value, raw_value, range )
    }

    pub fn new_text(value: &str, range: ASTRange ) -> Self {
        ASTElm::build( ASTType::Text, ASTMetaData::Nil, value, value, range )
    }

    pub fn new_emphasis( value: &str, raw_value: &str, range: ASTRange ) -> Self {
        ASTElm::build( ASTType::Emphasis, ASTMetaData::Nil, value, raw_value, range )
    }

    pub fn new_softbreak( range: ASTRange ) -> Self {
        ASTElm::build( ASTType::SoftBreak, ASTMetaData::Nil, "\n", "\n", range )
    }

    pub fn new_hardbreak( range: ASTRange ) -> Self {
        ASTElm::build( ASTType::HardBreak, ASTMetaData::Nil, "\n", "\n", range )
    }


}



#[derive(Debug, PartialEq, Serialize)]
pub enum ASTType {
    Document,
    Paragraph,
    Headers,
    Text,
    Emphasis,
    SoftBreak,
    HardBreak,
}

impl Default for ASTType {
    fn default() -> Self {
        ASTType::Document
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub enum ASTMetaData {
    Nil,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl Default for ASTMetaData {
    fn default() -> Self {
        ASTMetaData::Nil
    }
}

impl ASTNode {
    pub fn new(v: ASTElm) -> Self {
        ASTNode {
            data: v,
            children: vec![],
        }
    }

    pub fn append(&mut self, v: ASTElm) {
        let node = ASTNode::new(v);
        self.append_node(node);
    }

    pub fn append_node(&mut self, node: ASTNode) {
        let rc_node = Rc::new(RefCell::new(node));
        self.children.push(Rc::clone(&rc_node));
    }

    pub fn append_node_from_vec(&mut self, nodes: Vec<ASTNode>) {
        for node in nodes {
            self.append_node(node);
        }
    }

    //
    // --- setter, getter ---
    //

    pub fn node_type(&self) -> &ASTType {
        &self.data.elm_type
    }

    pub fn meta(&self) -> &ASTMetaData {
        &self.data.elm_meta
    }

    pub fn value(&self) -> &String {
        &self.data.value
    }

    pub fn raw_value(&self) -> &String {
        &self.data.raw_value
    }

    pub fn range(&self) -> &ASTRange {
        &self.data.range
    }

    pub fn node_type_mut(&mut self) -> &mut ASTType {
        &mut self.data.elm_type
    }

    pub fn meta_mut(&mut self) -> &mut ASTMetaData {
        &mut self.data.elm_meta
    }

    pub fn value_mut(&mut self) -> &mut String {
        &mut self.data.value
    }

    pub fn raw_value_mut(&mut self) -> &mut String {
        &mut self.data.raw_value
    }

    pub fn range_mut(&mut self) -> &mut ASTRange {
        &mut self.data.range
    }

    pub fn set_node_type(&mut self, v: ASTType) {
        *self.node_type_mut() = v;
    }

    pub fn set_meta(&mut self, v: ASTMetaData) {
        *self.meta_mut() = v;
    }

    pub fn set_value(&mut self, v: String) {
        *self.value_mut() = v;
    }

    pub fn set_raw_value(&mut self, v: String) {
        *self.raw_value_mut() = v;
    }

    pub fn set_range(&mut self, v: ASTRange) {
        *self.range_mut() = v;
    }

    //
    // --- rendering ---
    //
    pub fn render_debug_format(&self) -> String {
        self._render_debug_format(self)
    }

    fn _render_tag(&self, tagname: &str, node: &ASTNode) -> String {
        let mut result: String = "<".to_string() + tagname + ">";
        for child in &node.children {
            result = result + &self._render_debug_format(&child.borrow());
        }
        result = result + "</" + tagname + ">";
        return result;
    }

    fn _render_debug_format(&self, node: &ASTNode) -> String {
        let mut result: String = "".to_string();
        match node.node_type() {
            ASTType::Document => {
                result += &self._render_tag("document", &node);
            }
            ASTType::Paragraph => {
                result += &self._render_tag("paragraph", &node);
            }
            ASTType::Headers => {
                result += &self._render_tag("header", &node);
            }
            ASTType::Text => {
                result += &("<text>".to_string() + &node.value().to_string() + "</text>");
            }
            ASTType::Emphasis => {
                result += &self._render_tag("emphasis", &node);
            }
            ASTType::SoftBreak => {
                //result = node.value().to_string();
                result += "<softbreak />";
            }
            ASTType::HardBreak => {
                result += "<hardbreak />";
            }
            _ => {
                result = result + "<error>Undefined ASTNode Type</error>";
            }
        }
        return result;
    }
}

// イテレータの実装
/*
impl<T> Iterator for ASTNode<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {

    }
}
*/

/*
#[cfg(test)]
mod tests {
    use super::*;

    fn ASTNode_return() -> ASTNode {
        ASTNode::new( ASTElm {
            elm_type: ASTType::Document,
            elm_meta: ASTMetaData::Nil,
            value: "astnode_return".to_string(),
        })
    }

    fn ASTNode_markdown_header() -> ASTNode {

        let mut node = ASTNode::new( ASTElm {
            elm_type: ASTType::Headers,
            elm_meta: ASTMetaData::H1,
            value: "".to_string(),
        });

        node.append( ASTElm{
            elm_type: ASTType::Text,
            elm_meta: ASTMetaData::Nil,
            value: "Headering 1".to_string(),
        });

        return node
    }

    fn ASTNode_markdown_paragraph() -> ASTNode {

        let mut node = ASTNode::new( ASTElm {
            elm_type: ASTType::Paragraph,
            elm_meta: ASTMetaData::Nil,
            value: "".to_string(),
        });

        node.append( ASTElm{
            elm_type: ASTType::Text,
            elm_meta: ASTMetaData::Nil,
            value: "This is Paragraph block.".to_string(),
        });

        return node
    }

    #[test]
    fn ASTNode_basics() {
        let mut node = ASTNode::new( ASTElm{
            elm_type: ASTType::Document,
            elm_meta: ASTMetaData::Nil,
            value: "hello".to_string(),
        } );

        node.append( ASTElm{
            elm_type: ASTType::Document,
            elm_meta: ASTMetaData::Nil,
            value: "Hello2".to_string(),
        });

        node.append_node( ASTNode::new( ASTElm{
            elm_type: ASTType::Document,
            elm_meta: ASTMetaData::Nil,
            value: "hello3".to_string(),
        } ) );

        node.append_node( ASTNode_return() );

        println!("ASTNode_basics tests");
        println!("{:?}", node);
        println!("{:?}", node.node_type());
    }

    #[test]
    fn ASTNode_markdown() {
        let mut node_empty = ASTNode::new(Default::default());
        let mut node = ASTNode::new( ASTElm{
            elm_type: ASTType::Document,
            elm_meta: ASTMetaData::Nil,
            value: "# Headering 1\nThis is paragraph block.".to_string(),
        } );

        node.append_node( ASTNode_markdown_header() );
        node.append_node( ASTNode_markdown_paragraph() );

        println!("ASTNode_markdown tests");
        println!("{:?}", node);
        println!("{:?}", node_empty);
    }
}*/
