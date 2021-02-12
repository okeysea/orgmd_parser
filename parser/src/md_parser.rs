extern crate nom;
use crate::ast::*;

use nom::branch::{alt, permutation};
use nom::character::complete::{
    alpha1, anychar, char, digit1, line_ending, multispace0, not_line_ending, space0, space1,
};
use nom::combinator::{cond, eof, map, not, peek};
use nom::error::{ErrorKind, ParseError};
use nom::multi::{many0, many1, many_m_n};
use nom::sequence::{delimited, tuple};
use nom::{AsChar, Err, IResult, InputIter, InputLength, InputTakeAtPosition, Slice};
use std::ops::RangeFrom;

// 制御文字と半角スペース以外の文字 -> OK
fn decide_char_printable(input: char) -> Result<char, String> {
    let i = input as u32;
    let mut r = false;

    // 制御文字の判定
    r = 0x00 <= i && i <= 0x1F;
    r = r || i == 0x7F; // delete

    // 半角スペースの判定
    r = r || i == 0x20;

    if r == false {
        Ok(input)
    } else {
        Err("not printable char".to_string())
    }
}

// パーサコンビネータ用 制御文字と半角スペース以外の文字を受け入れる関数
fn printable_char<T, E: ParseError<T>>(input: T) -> IResult<T, char, E>
where
    T: InputIter + InputLength + Slice<RangeFrom<usize>>,
    <T as InputIter>::Item: AsChar,
    <T as InputIter>::Item: std::fmt::Debug,
{
    let mut it = input.iter_indices();
    match it.next() {
        None => Err(Err::Error(E::from_error_kind(input, ErrorKind::Eof))),
        Some((_, c)) => match it.next() {
            None => match decide_char_printable(c.as_char()) {
                Ok(c) => Ok((input.slice(input.input_len()..), c.as_char())),
                Err(_) => Err(Err::Error(E::from_error_kind(input, ErrorKind::Char))),
            },
            Some((idx, _)) => match decide_char_printable(c.as_char()) {
                Ok(c) => Ok((input.slice(idx..), c.as_char())),
                Err(_) => Err(Err::Error(E::from_error_kind(input, ErrorKind::Char))),
            },
        },
    }
}

struct Parser {
    current_pos: ASTPos,
    tran_buff: Vec<ASTPos>,
}

impl Parser {
    fn new() -> Self {
        Self {
            current_pos: ASTPos::new(0,0,0),
            tran_buff: vec![],
        }
    }

    // -- transactions --
    
    fn pos_begin_transaction(&mut self) {
        self.tran_buff.push( self.current_pos );
    }

    fn pos_commit(&mut self) {
        self.tran_buff.pop();
    }

    fn pos_rollback(&mut self) {
        match self.tran_buff.pop() {
            Some(result) => self.current_pos = result,
            None => {},
        }
    }

    // -- pos counter --
    
    fn increase_pos_n(&mut self, n: u32) {
        self.current_pos.increase_pos_n(n);
    }
    
    fn increase_line_n(&mut self, n: u32) {
        self.current_pos.increase_line_n(n);
    }

    fn increase_ch_n(&mut self, n: u32){
        self.current_pos.increase_ch_n(n);
    }

    fn increase_pos(&mut self) {
        self.increase_pos_n(1);
    }

    fn increase_line(&mut self, n: u32) {
        self.increase_line_n(1);
    }
    
    fn increase_ch(&mut self, n: u32) {
        self.increase_ch_n(1);
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

impl Parser {
    /* ------- characters ------- */
    // コントロール文字、スペース・タブ・改行を除くUTF-8文字すべてを受けいれる
    // not-control-char

    fn parse_nc_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_ {
        move |s| {
            printable_char(s)
        }
    }

    fn parse_tab(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('\t')(s)
        }
    }

    fn parse_space(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char(' ')(s)
        }
    }

    // コントロール文字・改行を除くUTF-8文字すべてを受けいれる
    // not-break-char
    fn parse_nbr_char(&self) -> impl Fn(&str) -> IResult<&str, String> + '_  {
        move |s| {
            map(
                alt((self.parse_tab(), self.parse_space(), self.parse_nc_char())),
                |input_s: char| {
                    let ret: String = input_s.to_string();
                    return ret;
                },
            )(s)
        }
    }

    fn parse_blank_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            alt((char(' '), char('\n')))(s)
        }
    }

    // 記号(ASCII)・コントロール文字除くUTF-8文字すべてを受けいれる
    // not-special-char
    // インライン書式のパース用→ *Empasis*
    fn parse_nsp_char(&self) -> impl Fn(&str) -> IResult<&str, String> + '_  {
        move |s| {
            map(
                tuple((peek(not(self.parse_sp_char())), self.parse_nbr_char())),
                |(_, r)| r,
            )(s)
        }
    }

    fn parse_sp_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            alt((
                    self.parse_exclamation_char(),
                    self.parse_double_quotation_char(),
                    self.parse_pound_char(),
                    self.parse_dollar_char(),
                    self.parse_percent_char(),
                    self.parse_ampersand_char(),
                    self.parse_single_quotation_char(),
                    self.parse_lparenthesis_char(),
                    self.parse_rparenthesis_char(),
                    self.parse_asterisk_char(),
                    self.parse_plus_char(),
                    self.parse_comma_char(),
                    self.parse_hyphen_char(),
                    self.parse_period_char(),
                    self.parse_slash_char(),
                    self.parse_colon_char(),
                    self.parse_semicolon_char(),
                    self.parse_lessthan_char(),
                    self.parse_equal_char(),
                    self.parse_greaterthan_char(),
                    alt((
                            self.parse_question_char(),
                            self.parse_at_char(),
                            self.parse_lsquare_char(),
                            self.parse_backslash_char(),
                            self.parse_rsquare_char(),
                            self.parse_hat_char(),
                            self.parse_underline_char(),
                            self.parse_backquote_char(),
                            self.parse_lcurly_char(),
                            self.parse_pipe_char(),
                            self.parse_rcurly_char(),
                            self.parse_tilde_char(),
                    )),
                    ))(s)
        }
    }

    fn parse_exclamation_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('!')(s)
        }
    }

    fn parse_double_quotation_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('"')(s)
        }
    }

    fn parse_pound_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('#')(s)
        }
    }

    fn parse_dollar_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('$')(s)
        }
    }

    fn parse_percent_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('%')(s)
        }
    }

    fn parse_ampersand_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('&')(s)
        }
    }

    fn parse_single_quotation_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('\'')(s)
        }
    }

    fn parse_lparenthesis_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('(')(s)
        }
    }

    fn parse_rparenthesis_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char(')')(s)
        }
    }

    fn parse_asterisk_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('*')(s)
        }
    }

    fn parse_plus_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('+')(s)
        }
    }

    fn parse_comma_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char(',')(s)
        }
    }

    fn parse_hyphen_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('-')(s)
        }
    }

    fn parse_period_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('.')(s)
        }
    }

    fn parse_slash_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('/')(s)
        }
    }

    fn parse_colon_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char(':')(s)
        }
    }

    fn parse_semicolon_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char(';')(s)
        }
    }

    fn parse_lessthan_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('<')(s)
        }
    }

    fn parse_equal_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('=')(s)
        }
    }

    fn parse_greaterthan_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('>')(s)
        }
    }

    fn parse_question_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('?')(s)
        }
    }

    fn parse_at_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('@')(s)
        }
    }

    fn parse_lsquare_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('[')(s)
        }
    }

    fn parse_backslash_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('\\')(s)
        }
    }

    fn parse_rsquare_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char(']')(s)
        }
    }

    fn parse_hat_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('^')(s)
        }
    }

    fn parse_underline_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('_')(s)
        }
    }

    fn parse_backquote_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('`')(s)
        }
    }

    fn parse_lcurly_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('{')(s)
        }
    }

    fn parse_pipe_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('|')(s)
        }
    }

    fn parse_rcurly_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('}')(s)
        }
    }

    fn parse_tilde_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('~')(s)
        }
    }
}

/* ---------- strings ----------*/

impl Parser {
    // ソフトブレイク
    fn parse_soft_break(&self) -> impl Fn(&str) -> IResult<&str, String> + '_  {
        move |s| {
            map(line_ending, |input_s: &str| input_s.to_owned())(s)
        }
    }

    fn parse_soft_break_node(&self) -> impl Fn(&str) -> IResult<&str, ASTNode> + '_  {
        move |s| {
            match self.parse_soft_break()(s) {
                Ok((remain, _)) => {
                    let node = ASTNode::new(ASTElm {
                        elm_type: ASTType::SoftBreak,
                        elm_meta: ASTMetaData::Nil,
                        value: "\n".to_string(),
                        raw_value: s.slice(..s.len() - remain.len()).to_string(),
                    });
                    Ok((remain, node))
                }
                Err(e) => Err(e),
            }
        }
    }

    // ハードブレイク(ブロック終了)
    fn parse_hard_break(&self) -> impl Fn(&str) -> IResult<&str, String> + '_  {
        move |s| {
            map(
                many_m_n(2, 9999, self.parse_soft_break()),
                |input_s: Vec<String>| input_s.into_iter().collect(),
            )(s)
        }
    }

    fn parse_break_or_eof(&self) -> impl Fn(&str) -> IResult<&str, String> + '_  {
        move |s| {
            alt((
                    self.parse_soft_break(),
                    map(eof, |input_s: &str| input_s.to_owned()),
            ))(s)
        }
    }

    fn parse_char(&self) -> impl Fn(&str) -> IResult<&str, String> + '_  {
        move |s| {
            alt((self.parse_soft_break(), self.parse_nbr_char()))(s)
        }
    }

    fn parse_nbr_string(&self) -> impl Fn(&str) -> IResult<&str, String> + '_  {
        move |s| {
            map(many1(self.parse_nbr_char()), |input_s: Vec<String>| {
                util_vecstring_to_string(input_s)
            })(s)
        }
    }

    fn parse_string(&self) -> impl Fn(&str) -> IResult<&str, String> + '_  {
        move |s| {
            map(many1(self.parse_char()), |input_s: Vec<String>| {
                util_vecstring_to_string(input_s)
            })(s)
        }
    }

    fn parse_nsp_string(&self) -> impl Fn(&str) -> IResult<&str, String> + '_  {
        move |s| {
            map(many1(self.parse_nsp_char()), |input_s: Vec<String>| {
                util_vecstring_to_string(input_s)
            })(s)
        }
    }
}

impl Parser {

    // インライン書式のパース
    fn parse_inline(&self) -> impl Fn(&str) -> IResult<&str, ASTNode> + '_  {
        move |s| {
            match alt((
                    map(self.parse_nsp_string(), |input_s: String| {
                        let node = ASTNode::new(ASTElm {
                            elm_type: ASTType::Text,
                            elm_meta: ASTMetaData::Nil,
                            raw_value: s.slice(..input_s.len()).to_string(),
                            value: input_s,
                        });
                        return node;
                    }),
                    self.parse_emphasis(),
            ))(s)
            {
                Ok((remain, node)) => Ok((remain, node)),
                Err(_) => Err(Err::Error(ParseError::from_error_kind(s, ErrorKind::Char))),
            }
        }
    }

    /*
     * 強調
     * (入れ子を許容)
     *
     * TODO: アスタリスク前後の記号の取り扱いをcommonmarkに合わせる
     * */
    fn parse_emphasis(&self) -> impl Fn(&str) -> IResult<&str, ASTNode> + '_  {
        move |s| {
            match delimited(
                tuple((char('*'), peek(not(self.parse_soft_break())))),
                many1(alt((self.parse_inline(), self.parse_soft_break_node()))),
                tuple((peek(not(self.parse_soft_break())), char('*'))),
            )(s)
            {
                Ok((remain, nodes)) => {
                    let mut node = ASTNode::new(ASTElm {
                        elm_type: ASTType::Emphasis,
                        elm_meta: ASTMetaData::Nil,
                        value: "".to_string(),
                        raw_value: s.slice(..s.len() - remain.len()).to_string(),
                    });
                    node.append_node_from_vec(nodes);
                    Ok((remain, node))
                }
                Err(_) => Err(Err::Error(ParseError::from_error_kind(s, ErrorKind::Char))),
            }
        }
    }

    /*
     * パラグラフ中のアスタリスクのパース
     * (強調構文から漏れたアスタリスクの処理)
     * */
    fn parse_sp_asterisk(&self) -> impl Fn(&str) -> IResult<&str, ASTNode> + '_  {
        move |s| {
            match map(self.parse_asterisk_char(), |input_s: char| {
                let node = ASTNode::new(ASTElm {
                    elm_type: ASTType::Text,
                    elm_meta: ASTMetaData::Nil,
                    value: input_s.to_string(),
                    raw_value: s.slice(..input_s.len()).to_string(),
                });
                return node;
            })(s)
            {
                Ok((remain, node)) => Ok((remain, node)),
                Err(_) => Err(Err::Error(ParseError::from_error_kind(s, ErrorKind::Char))),
            }
        }
    }

    /*
       <Headers>   ::=   <Space>{0..3} '#'{1..6} <Space>{1..} <NBRString> ['#'*] <BreakOrEof>
       | <Space>{0..3} ( ( <Char> | <Space> ) <SoftBreak> )* ('='|'-')*
       TODO: 上段の構文の後ろシャープ記号を無視するようにする
       TODO: 下段の構文のパース処理
       */
    fn parse_headers(&self) -> impl Fn(&str) -> IResult<&str, ASTNode> + '_  {
        move |s| {
            let r = tuple((
                    many_m_n(0, 3, self.parse_space()),
                    map(many_m_n(1, 6, char('#')), |input_s: Vec<char>| {
                        input_s.len()
                    }),
                    //space1,
                    many1(char(' ')),
                    self.parse_nbr_string(),
                    many0((char('#'))),
                    self.parse_break_or_eof(),
            ))(s);
            match r {
                Ok((remain, (_, level, _, text, _, _))) => {
                    let mut node = ASTNode::new(ASTElm {
                        elm_type: ASTType::Headers,
                        elm_meta: ASTMetaData::Nil,
                        raw_value: s.slice(..s.len() - remain.len()).to_string(),
                        ..Default::default()
                    });
                    // インライン書式のパース
                    // TODO: エラー処理
                    let (_, inline_node) = many1(self.parse_inline())(&text).unwrap();
                    node.append_node_from_vec(inline_node);
                    Ok((remain, node))
                }
                Err(_) => Err(Err::Error(ParseError::from_error_kind(s, ErrorKind::Char))),
            }
        }
    }

    /* 段落のパース
     *
     * 処理順序
     *   段落の切り出しー> インライン書式のパース
     * */
    fn parse_paragraph(&self) -> impl Fn(&str) -> IResult<&str, ASTNode> + '_  {
        move |s| {
            match self.parse_separate()(s) {
                Ok((remain, separated)) => {
                    let child_r = many1(alt((
                                self.parse_inline(),
                                self.parse_soft_break_node(),
                                self.parse_sp_asterisk(),
                    )))(&separated);

                    match child_r {
                        Ok((_, child_node)) => {
                            let mut node = ASTNode::new(ASTElm {
                                elm_type: ASTType::Paragraph,
                                elm_meta: ASTMetaData::Nil,
                                raw_value: separated.to_string(),
                                ..Default::default()
                            });
                            node.append_node_from_vec(child_node);
                            Ok((remain, node))
                        }
                        Err(_) => Err(Err::Error(ParseError::from_error_kind(s, ErrorKind::Char))),
                    }
                }
                Err(_) => Err(Err::Error(ParseError::from_error_kind(s, ErrorKind::Char))),
            }
        }
    }

    // 入力位置からブロック終了までの文字を返す
    fn parse_separate(&self) -> impl Fn(&str) -> IResult<&str, String> + '_  {
        move |s| {
            let r = tuple((self.parse_nbr_string(), self.parse_break_or_eof()))(s);
            match r {
                Ok((remain, (text, br))) => {
                    let quit = self.parse_separator()(remain);
                    match quit {
                        // 最終行は改行を出力しない
                        Ok((remain, _)) => Ok((remain, text)),
                        Err(_) => {
                            let continue_r = self.parse_separate()(remain);
                            match continue_r {
                                Ok((continue_remain, continue_text)) => {
                                    // 行中は改行を出力する
                                    Ok((continue_remain, text + &br + &continue_text))
                                }
                                Err(_) => Err(Err::Error(ParseError::from_error_kind(s, ErrorKind::Char))),
                            }
                        }
                    }
                }
                Err(_) => Err(Err::Error(ParseError::from_error_kind(s, ErrorKind::Char))),
            }
        }
    }

    // ブロック終了の判定
    // 終了判断: Headerの書式、ハードブレイク
    fn parse_separator(&self) -> impl Fn(&str) -> IResult<&str, String> + '_  {
        move |s| {
            let r = alt((
                    self.parse_hard_break(),
                    self.parse_break_or_eof(),
                    // 見出し(headering)
                    map(
                        tuple((
                                many_m_n(0, 3, self.parse_space()),
                                map(many_m_n(1, 6, char('#')), |input_s: Vec<char>| {
                                    input_s.len()
                                }),
                                many1(char(' ')),
                        )),
                        |input_s| "".to_string(),
                    ),
            ))(s);
            match r {
                Ok((_, parse_result)) => Ok((s.slice(parse_result.chars().count()..), "".to_string())),
                Err(_) => Err(Err::Error(ParseError::from_error_kind(s, ErrorKind::Char))),
            }
        }
    }

    fn parse_blocks(&self) -> impl Fn(&str) -> IResult<&str, ASTNode> + '_  {
        move |s| {
            alt((self.parse_headers(), self.parse_paragraph()))(s)
        }
    }

}

pub fn md_parse(s: &str, mut node: ASTNode) -> ASTNode {
    node.set_node_type(ASTType::Document);
    node.set_meta(ASTMetaData::Nil);
    node.set_value("".to_string());
    node.set_raw_value(s.to_string());
    
    let parser = Parser::new();

    let r = many0(parser.parse_blocks())(s);

    match r {
        Ok((remain, result)) => {
            for child_node in result {
                node.append_node(child_node);
            }
        }
        Err(_) => {}
    }

    return node;
}

/*
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
}*/
