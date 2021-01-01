extern crate nom;
use crate::module::ast::*;

use nom::{
    Err, IResult, InputIter, Slice, AsChar,
    InputTakeAtPosition, InputLength,
};
use nom::error::{
    ErrorKind, ParseError,
};
use nom::character::complete::{
    char, digit1, multispace0, line_ending,
    not_line_ending, space0, space1, anychar,
    alpha1,
};
use nom::branch::{
    alt, permutation,
};
use nom::combinator::{
    eof, map, peek, not,
    cond,
};
use nom::sequence::{
    delimited, tuple,
};
use nom::multi::{
    many_m_n, many1, many0,
};
use std::ops::RangeFrom;

fn decide_char_printable(input: char) -> Result<char, String>
{
    let i = input as u32;
    let mut r = false;
    
    // 制御文字の判定
    r = 0x00 <= i && i <= 0x1F;
    r = r || i == 0x7F; // delete

    // 半角スペースの判定
    r = r || i == 0x20;

    if r == false {
        Ok(input)
    }else{
        Err("not printable char".to_string())
    }
}

fn printable_char<T, E: ParseError<T>>(input: T) -> IResult<T, char, E>
where
    T: InputIter + InputLength + Slice<RangeFrom<usize>>,
    <T as InputIter>::Item: AsChar, <T as InputIter>::Item: std::fmt::Debug,
{
    let mut it = input.iter_indices();
    match it.next() {
        None => Err(Err::Error(E::from_error_kind(input, ErrorKind::Eof))),
        Some((_, c)) => match it.next() {
            None => {
                match decide_char_printable(c.as_char()) {
                    Ok(c)  => Ok((input.slice(input.input_len()..), c.as_char())),
                    Err(_) => Err(Err::Error(E::from_error_kind(input, ErrorKind::Char))),
                }
            },
            Some((idx, _)) => {
                match decide_char_printable(c.as_char()) {
                    Ok(c)  => Ok((input.slice(idx..), c.as_char())),
                    Err(_) => Err(Err::Error(E::from_error_kind(input, ErrorKind::Char))),
                }
            },
        }
    }
}

// UTILITIES

fn util_vecstr_to_str(s: Vec<&str>) -> String {
    s.concat()
}

fn util_vecstring_to_string(s: Vec<String>) -> String {
    s.into_iter().collect()
}

fn util_vecchar_to_string(s: Vec<char>) -> String {
    s.into_iter().collect()
}


/* ------- characters ------- */
// コントロール文字、スペース・タブ・改行を除くUTF-8文字すべてを受けいれる
// not-control-char
fn parse_nc_char(s: &str) -> IResult<&str, char> {
    printable_char(s)
}

fn parse_tab(s: &str) -> IResult<&str, char> {
    char('\t')(s)
}

fn parse_space(s: &str) -> IResult<&str, char> {
    char(' ')(s)
}

// コントロール文字・改行を除くUTF-8文字すべてを受けいれる
// not-break-char
fn parse_nbr_char(s: &str) -> IResult<&str, String> {
    map(alt((
        parse_tab,
        parse_space,
        parse_nc_char
    )), |input_s: char| {
        let ret: String = input_s.to_string();
        return ret
    })(s)
}

fn parse_blank_char(s: &str) -> IResult<&str, char> {
    alt((
        char(' '),
        char('\n'),
    ))(s)
}

// 記号(ASCII)・コントロール文字除くUTF-8文字すべてを受けいれる
// not-special-char
// インライン書式のパース用→ *Empasis*
fn parse_nsp_char(s: &str) -> IResult<&str, String> {
    map(tuple((
        peek(not(parse_sp_char)),
        parse_nbr_char,
    )),|(_, r)| r )(s)
}

fn parse_sp_char(s: &str) -> IResult<&str, char> {
    alt((
        parse_exclamation_char,
        parse_double_quotation_char,
        parse_pound_char,
        parse_dollar_char,
        parse_percent_char,
        parse_ampersand_char,
        parse_single_quotation_char,
        parse_lparenthesis_char,
        parse_rparenthesis_char,
        parse_asterisk_char,
        parse_plus_char,
        parse_comma_char,
        parse_hyphen_char,
        parse_period_char,
        parse_slash_char,
        parse_colon_char,
        parse_semicolon_char,
        parse_lessthan_char,
        parse_equal_char,
        parse_greaterthan_char,
        alt((parse_question_char,
        parse_at_char,
        parse_lsquare_char,
        parse_backslash_char,
        parse_rsquare_char,
        parse_hat_char,
        parse_underline_char,
        parse_backquote_char,
        parse_lcurly_char,
        parse_pipe_char,
        parse_rcurly_char,
        parse_tilde_char,
        ))
    ))(s)
}

fn parse_exclamation_char(s: &str) -> IResult<&str, char>{
    char('!')(s)
}

fn parse_double_quotation_char(s: &str) -> IResult<&str, char>{
    char('"')(s)
}

fn parse_pound_char(s: &str) -> IResult<&str, char>{
    char('#')(s)
}

fn parse_dollar_char(s: &str) -> IResult<&str, char>{
    char('$')(s)
}

fn parse_percent_char(s: &str) -> IResult<&str, char>{
    char('%')(s)
}

fn parse_ampersand_char(s: &str) -> IResult<&str, char>{
    char('&')(s)
}

fn parse_single_quotation_char(s: &str) -> IResult<&str, char>{
    char('\'')(s)
}

fn parse_lparenthesis_char(s: &str) -> IResult<&str, char>{
    char('(')(s)
}

fn parse_rparenthesis_char(s: &str) -> IResult<&str, char>{
    char(')')(s)
}

fn parse_asterisk_char(s: &str) -> IResult<&str, char>{
    char('*')(s)
}

fn parse_plus_char(s: &str) -> IResult<&str, char>{
    char('+')(s)
}

fn parse_comma_char(s: &str) -> IResult<&str, char>{
    char(',')(s)
}

fn parse_hyphen_char(s: &str) -> IResult<&str, char>{
    char('-')(s)
}

fn parse_period_char(s: &str) -> IResult<&str, char>{
    char('.')(s)
}

fn parse_slash_char(s: &str) -> IResult<&str, char>{
    char('/')(s)
}

fn parse_colon_char(s: &str) -> IResult<&str, char>{
    char(':')(s)
}

fn parse_semicolon_char(s: &str) -> IResult<&str, char>{
    char(';')(s)
}

fn parse_lessthan_char(s: &str) -> IResult<&str, char>{
    char('<')(s)
}

fn parse_equal_char(s: &str) -> IResult<&str, char>{
    char('=')(s)
}

fn parse_greaterthan_char(s: &str) -> IResult<&str, char>{
    char('>')(s)
}

fn parse_question_char(s: &str) -> IResult<&str, char>{
    char('?')(s)
}

fn parse_at_char(s: &str) -> IResult<&str, char>{
    char('@')(s)
}

fn parse_lsquare_char(s: &str) -> IResult<&str, char>{
    char('[')(s)
}

fn parse_backslash_char(s: &str) -> IResult<&str, char>{
    char('\\')(s)
}

fn parse_rsquare_char(s: &str) -> IResult<&str, char>{
    char(']')(s)
}

fn parse_hat_char(s: &str) -> IResult<&str, char>{
    char('^')(s)
}

fn parse_underline_char(s: &str) -> IResult<&str, char>{
    char('_')(s)
}

fn parse_backquote_char(s: &str) -> IResult<&str, char>{
    char('`')(s)
}

fn parse_lcurly_char(s: &str) -> IResult<&str, char>{
    char('{')(s)
}

fn parse_pipe_char(s: &str) -> IResult<&str, char>{
    char('|')(s)
}

fn parse_rcurly_char(s: &str) -> IResult<&str, char>{
    char('}')(s)
}

fn parse_tilde_char(s: &str) -> IResult<&str, char>{
    char('~')(s)
}

/* ---------- strings ----------*/

// ソフトブレイク
fn parse_soft_break(s: &str) -> IResult<&str, String> {
    map(line_ending, |input_s: &str| input_s.to_owned() )(s)
}

fn parse_soft_break_node(s: &str) -> IResult<&str, ASTNode> {
    match parse_soft_break(s) {
        Ok((remain, _ )) => {
            let node = ASTNode::new( ASTElm {
                elm_type: ASTType::SoftBreak,
                elm_meta: ASTMetaData::Nil,
                value: "\n".to_string(),
                raw_value: s.slice(..s.len() - remain.len()).to_string(),   
            });
            Ok((remain, node))
        },
        Err(e) => {
            Err(e)
        }
    }
}

// ハードブレイク(ブロック終了)
fn parse_hard_break(s: &str) -> IResult<&str, String> {
    map(many_m_n(2, 9999, parse_soft_break), |input_s: Vec<String>| {
        input_s.into_iter().collect() 
    })(s)
}

fn parse_break_or_eof(s: &str) -> IResult<&str, String> {
    alt((
        parse_soft_break,
        map(eof, |input_s: &str| input_s.to_owned()),
    ))(s)
}

fn parse_char(s: &str) -> IResult<&str, String> {
    alt((
        parse_soft_break,
        parse_nbr_char,
    ))(s)
}

fn parse_nbr_string(s: &str) -> IResult<&str, String> {
    map(many1(parse_nbr_char),|input_s: Vec<String>|{
        util_vecstring_to_string(input_s)
    })(s)
}

fn parse_string(s: &str) -> IResult<&str, String> {
    map(many1(parse_char),|input_s: Vec<String>|{
        util_vecstring_to_string(input_s)
    })(s)
}

fn parse_nsp_string(s: &str) -> IResult<&str, String> {
    map(many1(parse_nsp_char),|input_s: Vec<String>|{
        util_vecstring_to_string(input_s)
    })(s)
}

// インライン書式のパース
fn parse_inline(s: &str) -> IResult<&str, ASTNode> {
    println!("parse_inline(in): {:?}", s);
    match alt((
                map(parse_nsp_string, |input_s: String|{
                    let node = ASTNode::new( ASTElm {
                        elm_type: ASTType::Text,
                        elm_meta: ASTMetaData::Nil,
                        raw_value: s.slice(..input_s.len()).to_string(),
                        value: input_s,
                    });
                    return node
                }),
                parse_emphasis,
    ))(s){
        Ok((remain, node)) => {
            println!("parse_inline(Success): {:?}", s);
            Ok((remain, node))
        },
        Err(_)=>{
            println!("parse_inline(Error): {:?}", s);
            Err(Err::Error(ParseError::from_error_kind(s, ErrorKind::Char)))
        },
    }
}

/* 
 * 強調
 * (入れ子を許容)
 * */
fn parse_emphasis(s: &str) -> IResult<&str, ASTNode>{
    println!("parse_emphasis(in): {:?}", s);
    match delimited(
        tuple((char('*'), peek(not(parse_soft_break)))),
        many1(alt((
            parse_inline,
            parse_soft_break_node,
        ))),
        tuple((peek(not(parse_soft_break)),char('*'))),
    )(s){
        Ok((remain, nodes)) => {
            println!("parse_emphasis(Success): {:?} -> {:?}", s, remain);
            let mut node = ASTNode::new( ASTElm {
                elm_type: ASTType::Emphasis,
                elm_meta: ASTMetaData::Nil,
                value: "".to_string(),
                raw_value: s.slice(..s.len()-remain.len()).to_string(),
            });
            node.append_node_from_vec(nodes);   
            Ok((remain, node))
        },
        Err(_) => {
            println!("parse_emphasis(Error): {:?}", s);
            Err(Err::Error(ParseError::from_error_kind(s, ErrorKind::Char)))
        }
    }
}

/*
 * パラグラフ中のアスタリスクのパース
 * */
fn parse_sp_asterisk(s: &str) -> IResult<&str, ASTNode>{
    match map(parse_asterisk_char, |input_s: char|{
        let node = ASTNode::new( ASTElm {
            elm_type: ASTType::Text,
            elm_meta: ASTMetaData::Nil,
            value: input_s.to_string(),
            raw_value: s.slice(..input_s.len()).to_string(),
        });
        return node
    })(s){
        Ok((remain, node)) => {
            Ok((remain, node))
        },
        Err(_) => {
            Err(Err::Error(ParseError::from_error_kind(s, ErrorKind::Char)))
        }
    }
}

fn numeric_in_parantheses(s: &str) -> IResult<&str, &str> {
    delimited(
        char('('),
        delimited(multispace0, digit1, multispace0),
        char(')'),
    )(s)
}

/*
<Headers>   ::=   <Space>{0..3} '#'{1..6} <Space>{1..} <NBRString> ['#'*] <BreakOrEof>
                | <Space>{0..3} ( ( <Char> | <Space> ) <SoftBreak> )* ('='|'-')*
 */

fn parse_headers(s: &str) -> IResult<&str, ASTNode> {
    let r = tuple((
                many_m_n(0, 3, parse_space),
                map(many_m_n(1, 6, char('#')), |input_s: Vec<char>|{
                    input_s.len()
                }),
                //space1,
                many1(char(' ')),
                parse_nbr_string,
                many0((char('#'))),
                parse_break_or_eof,
            ))(s);
    match r {
        Ok((remain, (_, level, _, text, _, _))) => {
            let mut node = ASTNode::new( ASTElm {
                elm_type: ASTType::Headers,
                elm_meta: ASTMetaData::Nil,
                ..Default::default()
            });
            // インライン書式のパース
            // TODO: エラー処理
            let (_, inline_node) = many1(parse_inline)(&text).unwrap();
            node.append_node_from_vec( inline_node );
            Ok((remain, node))
        },
        Err(_) => {
            Err(Err::Error(ParseError::from_error_kind(s, ErrorKind::Char)))
        },
    }
}


fn parse_paragraph(s: &str) -> IResult<&str, ASTNode> {

    match parse_separate(s) {
        Ok((remain, separated)) => {

            let child_r = many1(
                alt((
                    parse_inline,
                    parse_soft_break_node,
                    parse_sp_asterisk,
                ))
            )(&separated);

            match child_r {
                Ok((_, child_node)) => {
                    let mut node = ASTNode::new( ASTElm {
                        elm_type: ASTType::Paragraph,
                        elm_meta: ASTMetaData::Nil,
                        raw_value: separated.to_string(),
                        ..Default::default()
                    });
                    node.append_node_from_vec( child_node );
                    Ok((remain, node))
                },
                Err(_) => {
                    Err(Err::Error(ParseError::from_error_kind(s, ErrorKind::Char)))
                },
            }
        },
        Err(_) => {
            Err(Err::Error(ParseError::from_error_kind(s, ErrorKind::Char)))
        }
    }
}

// 入力位置からブロック終了までの文字を返す
fn parse_separate(s: &str) -> IResult<&str, String>{
    let r = tuple((
                parse_nbr_string,
                parse_break_or_eof,
            ))(s);
    match r {
        Ok((remain, ( text, br ))) => {
            let quit = parse_separator(remain);
            match quit {
                // 最終行は改行を出力しない
                Ok((remain, _)) => Ok((remain, text)),                          
                Err(_) => {
                    let continue_r = parse_separate(remain);
                    match continue_r {
                        Ok((continue_remain, continue_text)) => {
                            // 行中は改行を出力する
                            Ok((continue_remain, text + &br + &continue_text)) 
                        },
                        Err(_) =>{
                            Err(Err::Error(ParseError::from_error_kind(s, ErrorKind::Char)))
                        }
                    }
                },
            }
        },
        Err(_) =>{
            Err(Err::Error(ParseError::from_error_kind(s, ErrorKind::Char)))
        }
    }
}

// ブロック終了の判定
// Headerの書式、ハードブレイク
fn parse_separator(s: &str) -> IResult<&str, String>{
    let r = alt((
                parse_hard_break,
                parse_break_or_eof,

                // 見出し(headering)
                map(tuple((
                    many_m_n(0, 3, parse_space),
                    map(many_m_n(1, 6, char('#')), |input_s: Vec<char>|{
                        input_s.len()
                    }),
                    many1(char(' '))
                )),|input_s| "".to_string()),
            ))(s);
    match r {
        Ok((_, parse_result)) => {
            Ok((s.slice(parse_result.chars().count()..), "".to_string()))
        },
        Err(_) => {
            Err(Err::Error(ParseError::from_error_kind(s, ErrorKind::Char)))
        },
    }
}


fn parse_blocks(s: &str) -> IResult<&str, ASTNode> {
    alt((
        parse_headers,
        parse_paragraph
    ))(s)
}

pub fn md_parse(s: &str, mut node: ASTNode) -> ASTNode {
    node.set_node_type( ASTType::Document );
    node.set_meta( ASTMetaData::Nil );
    node.set_value("".to_string());
    node.set_raw_value( s.to_string() );

    let r = many0( parse_blocks )(s);

    match r {
        Ok((remain, result)) => {
            for child_node in result {
                node.append_node(child_node);
            }
        },
        Err(_) => {
        }
    }

    return node
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decide_char_printable(){
        // println!("------------------------------------------\n");
        // println!("ascii 'a': {:?}", decide_char_printable('a'));
        // println!("ascii '\\n': {:?}", decide_char_printable('\n'));
        // println!("ascii ' ': {:?}", decide_char_printable(' '));

    }

    #[test]
    fn test_parse_nc_char(){
        // println!("------------------------------------------\n");
        // println!("{:?}",parse_nc_char("abc"));
        // println!("{:?}",parse_nc_char("\nabc"));
        // println!("{:?}",parse_nc_char("\tcde"));
    }

    #[test]
    fn test_parse_nbr_string(){
        let mut node = ASTNode::new( ASTElm {
            elm_type: ASTType::Text,
            elm_meta: ASTMetaData::Nil,
            value: "parse_nbr_string".to_string(),
        });
        // assert_eq!(parse_nbr_string("parse_nbr_string\ntest"), Ok(("\ntest", &node)));
    }

    #[test]
    fn test_parse_string(){
        let test_case = "parse_string\ntest\nallow line-break";
        assert_eq!(parse_string(test_case), Ok(("", test_case.to_string())));
    }
    
    #[test]
    fn test_parse_headers(){
        let mut dest = ASTNode::new( ASTElm {
            elm_type: ASTType::Headers,
            elm_meta: ASTMetaData::Nil,
            ..Default::default()
        });
        dest.append_node( ASTNode::new( ASTElm {
            elm_type: ASTType::Text,
            elm_meta: ASTMetaData::Nil,
            value: "headering".to_string()
        }) );
        assert_eq!(parse_headers("# headering"), Ok(("", dest)));

        let mut dest = ASTNode::new( ASTElm {
            elm_type: ASTType::Headers,
            elm_meta: ASTMetaData::Nil,
            ..Default::default()
        });
        dest.append_node( ASTNode::new( ASTElm {
            elm_type: ASTType::Text,
            elm_meta: ASTMetaData::Nil,
            value: "headering".to_string()
        }) );
        assert_eq!(parse_headers("# headering\nparagraph"), Ok(("paragraph", dest)));
    }

    #[test]
    fn test_permutation(){
        // println!("+++++++++++++++++++++++++++++++++++++++++++++");
        let r = permutation((
                many_m_n(0, 3, parse_space),
                map(many_m_n(1, 6, char('#')), |input_s: Vec<char>|{
                    input_s.len()
                }),
                space1,
                parse_nbr_string,
                many0((char('#'))),
                parse_break_or_eof,
        ))("### abc #");
        // println!("{:?}", r);
        match r {
            Ok((remain, (_, level, _, text, _, _))) => {
                let mut node = ASTNode::new( ASTElm {
                    elm_type: ASTType::Headers,
                    elm_meta: ASTMetaData::Nil,
                    ..Default::default()
                });
                // インライン書式のパース
                // TODO: エラー処理
                let (_, inline_node) = parse_inline(&text).unwrap();
                node.append_node( inline_node );
            },
            Err(_) => {
            },
        }
        // println!("+++++++++++++++++++++++++++++++++++++++++++++");
    }

    #[test]
    fn test_parse_blocks() {
        // println!("-----parse_blocks------");
        // println!("parse_blocks # headering -> {:?}", parse_blocks("# headering"));
        // println!("parse_blocks paragraph -> {:?}", parse_blocks("paragraph"));
        // println!("parse_blocks # headering\\nparagraph ->{:?}", parse_blocks("# headering\nparagraph"));
        // println!("parse_blocks # headering\\nparagraph\\n# headering2 ->{:?}", parse_blocks("# headering\nparagraph\n# headering2"));
        // println!("-----parse_blocks------");
    }

    #[test]
    fn test_md_parse() {
        let mut node = ASTNode::new( ASTElm{ ..Default::default() } );
        node = md_parse("# headering\nparagraph\n# headering2", node);

        let mut dest = ASTNode::new( ASTElm{ 
            elm_type: ASTType::Document,
            elm_meta: ASTMetaData::Nil,
            value: "".to_string(),
        });

        let mut headering = ASTNode::new( ASTElm{
            elm_type: ASTType::Headers,
            elm_meta: ASTMetaData::Nil,
            value: "".to_string(),
        });

        let mut paragraph = ASTNode::new( ASTElm{
            elm_type: ASTType::Paragraph,
            elm_meta: ASTMetaData::Nil,
            value: "".to_string(),
        });

        let mut headering2 = ASTNode::new( ASTElm{
            elm_type: ASTType::Headers,
            elm_meta: ASTMetaData::Nil,
            value: "".to_string(),
        });

        let mut text_headering = ASTNode::new( ASTElm{
            elm_type: ASTType::Text,
            elm_meta: ASTMetaData::Nil,
            value: "headering".to_string(),
        });

        let mut text_headering2 = ASTNode::new( ASTElm{
            elm_type: ASTType::Text,
            elm_meta: ASTMetaData::Nil,
            value: "headering2".to_string(),
        });

        let mut text_paragraph = ASTNode::new( ASTElm{
            elm_type: ASTType::Text,
            elm_meta: ASTMetaData::Nil,
            value: "paragraph".to_string(),
        });

        headering.append_node( text_headering );
        headering2.append_node( text_headering2 );
        paragraph.append_node( text_paragraph );
        dest.append_node( headering );
        dest.append_node( paragraph );
        dest.append_node( headering2 );

        assert_eq!(node, dest);


        // println!("+++md_parse++++++++++++++++++++++++++++++++++");
        // println!("{:?}", node);
        // println!("+++d_parse++++++++++++++++++++++++++++++++++");
    }

    #[test]
    fn test_parse_separate(){
        println!("!!!!! test parse separate !!!!!");
        println!("{:?}", parse_separate("paragraph\n# headering"));
        println!("{:?}", parse_separate("paragraph\n\n\nparagraph2"));
        println!("{:?}", parse_separate("paragraph"));
        println!("{:?}", parse_separate("paragraph\ncontinue_paragraph\n\nother_paragraph"));
        println!("{:?}", parse_separate("hogehoge\ncontinue_paragraph\nthird_paragraph\nfour_paragraph2"));
        println!("{:?}", parse_separate("日本語でも\nしっかりと\n動作するか\n確認します。令和\n\n"));
        println!("!!!!! test parse separate !!!!!");
    }
    
    /*
    #[test]
    fn test_permutation_spec(){
        // println!("permutation spec --------------------------------");
        
        let r = permutation((
                map( char('a'), |input_s: char| input_s.to_string()),
                map( char('b'), |input_s: char| input_s.to_string()),
                map( char('c'), |input_s: char| input_s.to_string()),
        ))("abc");
       
        // println!("{:?}", r);
        match r {
            Ok((remain, _)) => {
            },
            Err(_) => {
            },
        }
        // println!("permutation spec --------------------------------");
    }*/

    #[test]
    fn test_parse_nsp_char(){
        println!("{:?}", parse_nsp_char("abc"));
        println!("{:?}", parse_nsp_char("*abc"));
        println!("{:?}", parse_nsp_char("*bc"));
    }

    #[test]
    fn test_parse_nsp_string(){
        println!("{:?}", parse_nsp_string("hogehoge*hogehoge"));
        println!("{:?}", parse_nsp_string("*remain"));
        println!("{:?}", parse_nsp_string("a*hoge"));
        println!("{:?}", parse_nsp_string("hoge*"));
    }
}
