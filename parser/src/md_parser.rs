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
use nom::Parser as NomTParser;
use std::ops::RangeFrom;
use std::rc::Rc;
use std::cell::RefCell;

use log::{info, trace, warn, debug};

struct Parser {
    current_pos: RefCell<ASTPos>,
    previous_pos: RefCell<ASTPos>,
    tran_buff: RefCell<Vec<ASTPos>>,
    pos_lock: RefCell<bool>,
    pos_br: RefCell<bool>,
}

impl Parser {
    fn new() -> Self {
        Self {
            current_pos: RefCell::new(ASTPos::new(1,1,0)),
            previous_pos: RefCell::new(ASTPos::new(1,1,0)),
            tran_buff: RefCell::new(vec![]),
            pos_lock: RefCell::new(false),
            pos_br: RefCell::new(false),
        }
    }

    /* 
     * 使い方
     * 1. pos_begin_transaction() -> 呼び出された時点の位置情報をスタックに積む
     * 2. パース処理が成功 -> pos_commit() -> スタックからpopし破棄
     *    パース処理が失敗 -> pos_rollback() -> スタックに詰まれた位置情報を復元
     * 
     * パーサコンビネータによる解析ではバックトラックが発生するため巻き戻し処理が必要。
     * */

    fn pos_tran_depth(&self) -> usize {
        self.tran_buff.borrow().len()
    }

    fn pos_begin_transaction(&self) {
        if !*self.pos_lock.borrow() {
            self.tran_buff.borrow_mut().push( self.current_pos.borrow().clone() );

            /* for debug */
            let depth = self.pos_tran_depth();
            debug!("{empty:>width$}transaction({}): BEGIN {:?}", depth, self.current_pos, width=depth*2, empty="");
        }
    }

    fn pos_commit(&self) {
        if !*self.pos_lock.borrow() {

            /* for debug */
            let depth = self.pos_tran_depth();
            debug!("{empty:>width$}transaction({}): COMMIT {:?}", depth, self.current_pos, width=depth*2, empty="");

            self.tran_buff.borrow_mut().pop();
        }
    }

    fn pos_rollback(&self) {
        if !*self.pos_lock.borrow() {

            /* for debug */
            let depth = self.pos_tran_depth();

            match self.tran_buff.borrow_mut().pop() {
                Some(result) => { 
                    self.current_pos.borrow().set_pos( result.pos() );
                    self.current_pos.borrow().set_line( result.line() );
                    self.current_pos.borrow().set_ch( result.ch() );
                    debug!("{empty:>width$}transaction({}): ROLLBACK {:?}", depth, self.current_pos, width=depth*2, empty="");
                },
                None => {},
            }
        }
    }

    fn pos_get_range(&self) -> ASTRange {
        match self.tran_buff.borrow().last() {
            Some(begin) => {
                let end = self.current_pos.borrow();
                ASTRange {
                    begin: begin.clone(),
                    end: end.clone(),
                }
            },
            None => {
                let end = self.current_pos.borrow();
                ASTRange {
                    begin: ASTPos::new(1,1,0),
                    end: end.clone(),
                }
            },
        }
    }

    fn pos_get_previous_line_range(&self) -> ASTRange {
        match self.tran_buff.borrow().last() {
            Some(begin) => {
                let end = if *self.pos_br.borrow() { self.previous_pos.borrow() } else { self.current_pos.borrow() };
                ASTRange {
                    begin: begin.clone(),
                    end: end.clone(),
                }
            },
            None => {
                let end = self.current_pos.borrow();
                ASTRange {
                    begin: ASTPos::new(1,1,0),
                    end: end.clone(),
                }
            },
        }
    }

    /*
     * そもそも位置情報の更新が不要な解析処理(単に処理の関係で切り出すなど)ではロックを行う。
     */
    fn pos_lock(&self) {
        /* for debug */
        let depth = self.pos_tran_depth();
        debug!("{empty:>width$}POS({}): LOCK", depth, width=depth*2, empty="");

        *self.pos_lock.borrow_mut() = true;
    }

    fn pos_unlock(&self) {
        /* for debug */
        let depth = self.pos_tran_depth();
        debug!("{empty:>width$}POS({}): UNLOCK", depth, width=depth*2, empty="");

        *self.pos_lock.borrow_mut() = false;
    }

    // -- pos counter --
    fn be_change_pos(&self) {
        let pre_pos = self.previous_pos.borrow_mut();
        pre_pos.set_line( self.current_pos.borrow().line() );
        pre_pos.set_ch( self.current_pos.borrow().ch() );
        pre_pos.set_pos( self.current_pos.borrow().pos() );
        *self.pos_br.borrow_mut() = false;
    }

    fn increase_pos_n(&self, n: u32) {
        if !*self.pos_lock.borrow() {
            self.be_change_pos();
            self.current_pos.borrow_mut().increase_pos_n(n);
        }
    }
    
    fn increase_line_n(&self, n: u32) {
        if !*self.pos_lock.borrow() {
            self.be_change_pos();
            *self.pos_br.borrow_mut() = true;
            self.current_pos.borrow_mut().increase_line_n(n);
        }
    }

    fn increase_ch_n(&self, n: u32){
        if !*self.pos_lock.borrow() {
            self.be_change_pos();
            self.current_pos.borrow_mut().increase_ch_n(n);
        }
    }

    fn increase_pos(&self) {
        self.increase_pos_n(1);
    }

    fn increase_line(&self) {
        self.increase_line_n(1);
    }
    
    fn increase_ch(&self) {
        self.increase_ch_n(1);
    }

    fn pos_print_debug(&self) {
        println!("{:?}", &self.current_pos);
    }

    /*
     * パーサーごとに位置情報のロールバックをいちいち書いていられないので
     * ロールバックを簡素にするためにパーサーをラップして透過的なクロージャーを返す
     *  ※ マクロを用意した
     */
    fn parse_with_tran<'a, I, O, E: ParseError<I>, F>(&'a self, mut parsefn: F) -> impl FnMut(I) -> IResult<I, O, E> + 'a
        where
            F: NomTParser<I, O, E> + 'a
    {
        move |input: I| {
            self.pos_begin_transaction();
            match parsefn.parse(input) {
                Ok(r) => {
                    self.pos_commit();
                    Ok(r)
                },
                Err(e) => {
                    self.pos_rollback();
                    Err(e)
                }
            }
        }
    }

    fn parse_isolate<'a, I, O, E: ParseError<I>, F>(&'a self, mut parsefn: F) -> impl FnMut(I) -> IResult<I, O, E> + 'a
        where
            F: NomTParser<I, O, E> + 'a
    {
        move |input: I| {
            self.pos_lock();
            match parsefn.parse(input) {
                Ok(r) => {
                    self.pos_unlock();
                    Ok(r)
                },
                Err(e) => {
                    self.pos_unlock();
                    Err(e)
                }
            }
        }
    }
}

/*
 * 使い方
 * with_tran!(self, parser() )(s)
 *
 * 使い方の指針
 * バックトラックが発生するもの→全体を囲む
 */
macro_rules! with_tran {
    ($self:ident, $f: expr) => {
        $self.parse_with_tran($f)
    };

    // タプル用
    ($self:ident, $( $f: expr ),*) => {
        ($( $self.parse_with_tran($f), )*)
    };
}

/* 
 * パースには必要だが、パース時点では消費しないパーサを囲む(e.g. peek())
 * 間違いのもとなので、peek関数をラップするほうが良い？
 */
macro_rules! isolate {
    ($self:ident, $f: expr) => {
        $self.parse_isolate($f)
    };

    // タプル用
    ($self:ident, $( $f: expr ),*) => {
        ($( $self.parse_isolate($f), )*)
    };
}

// base char Parsers
impl Parser {
    // 制御文字と半角スペース以外の文字 -> OK
    fn decide_char_printable(&self) -> impl Fn(char) -> Result<char, String> {
        move |input| {
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
    }

    // パーサコンビネータ用 制御文字と半角スペース以外の文字を受け入れる関数
    fn printable_char<T, E: ParseError<T>>(&self) -> impl Fn(T) -> IResult<T, char, E> + '_
    where
        T: InputIter + InputLength + Slice<RangeFrom<usize>>,
        <T as InputIter>::Item: AsChar,
        <T as InputIter>::Item: std::fmt::Debug,
    {
        move |input: T| {
            let mut it = input.iter_indices();
            match it.next() {
                None => Err(Err::Error(E::from_error_kind(input, ErrorKind::Eof))),
                Some((_, c)) => match it.next() {
                    None => match self.decide_char_printable()(c.as_char()) {
                        Ok(c) => {
                            // 一文字加算
                            self.increase_ch();

                            /* for debug */
                            let depth = self.pos_tran_depth();
                            debug!("{empty:>width$}POS({}): INCREASE CHAR '{}'", depth, c, width=depth*2, empty="");

                            Ok((input.slice(input.input_len()..), c.as_char()))
                        },
                        Err(_) => Err(Err::Error(E::from_error_kind(input, ErrorKind::Char))),
                    },
                    Some((idx, _)) => match self.decide_char_printable()(c.as_char()) {
                        Ok(c) => {
                            // 一文字加算
                            self.increase_ch();
                            
                            /* for debug */
                            let depth = self.pos_tran_depth();
                            debug!("{empty:>width$}POS({}): INCREASE CHAR '{}'", depth, c, width=depth*2, empty="");

                            Ok((input.slice(idx..), c.as_char()))
                        },
                        Err(_) => Err(Err::Error(E::from_error_kind(input, ErrorKind::Char))),
                    },
                },
            } 
        }
    }

    fn single_char(&self, c: char) -> impl Fn(&str) -> IResult<&str, char> + '_{
        move |s| {
            match char(c)(s) {
                Ok((remain, result)) => {

                    /* for debug */
                    let depth = self.pos_tran_depth();
                    debug!("{empty:>width$}POS({}): INCREASE CHAR '{}'", depth, result, width=depth*2, empty="");

                    self.increase_ch();
                    Ok((remain, result))
                },
                Err(e) => Err(e)
            }
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

impl Parser {
    /* ------- characters ------- */
    // コントロール文字、スペース・タブ・改行を除くUTF-8文字すべてを受けいれる
    // not-control-char

    fn parse_nc_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_ {
        move |s| {
            self.printable_char()(s)
        }
    }

    fn parse_tab(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            char('\t')(s)
        }
    }

    fn parse_space(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char(' ')(s)
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
            alt((self.single_char(' '), self.single_char('\n')))(s)
        }
    }

    // 記号(ASCII)・コントロール文字除くUTF-8文字すべてを受けいれる
    // not-special-char
    // インライン書式のパース用→ *Empasis*
    fn parse_nsp_char(&self) -> impl Fn(&str) -> IResult<&str, String> + '_  {
        move |s| {
            map(
                tuple((isolate!(self, peek(not(self.parse_sp_char()))), self.parse_nbr_char())),
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
            self.single_char('!')(s)
        }
    }

    fn parse_double_quotation_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char('"')(s)
        }
    }

    fn parse_pound_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char('#')(s)
        }
    }

    fn parse_dollar_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char('$')(s)
        }
    }

    fn parse_percent_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char('%')(s)
        }
    }

    fn parse_ampersand_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char('&')(s)
        }
    }

    fn parse_single_quotation_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char('\'')(s)
        }
    }

    fn parse_lparenthesis_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char('(')(s)
        }
    }

    fn parse_rparenthesis_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char(')')(s)
        }
    }

    fn parse_asterisk_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char('*')(s)
        }
    }

    fn parse_plus_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char('+')(s)
        }
    }

    fn parse_comma_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char(',')(s)
        }
    }

    fn parse_hyphen_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char('-')(s)
        }
    }

    fn parse_period_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char('.')(s)
        }
    }

    fn parse_slash_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char('/')(s)
        }
    }

    fn parse_colon_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char(':')(s)
        }
    }

    fn parse_semicolon_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char(';')(s)
        }
    }

    fn parse_lessthan_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char('<')(s)
        }
    }

    fn parse_equal_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char('=')(s)
        }
    }

    fn parse_greaterthan_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char('>')(s)
        }
    }

    fn parse_question_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char('?')(s)
        }
    }

    fn parse_at_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char('@')(s)
        }
    }

    fn parse_lsquare_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char('[')(s)
        }
    }

    fn parse_backslash_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char('\\')(s)
        }
    }

    fn parse_rsquare_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char(']')(s)
        }
    }

    fn parse_hat_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char('^')(s)
        }
    }

    fn parse_underline_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char('_')(s)
        }
    }

    fn parse_backquote_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char('`')(s)
        }
    }

    fn parse_lcurly_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char('{')(s)
        }
    }

    fn parse_pipe_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char('|')(s)
        }
    }

    fn parse_rcurly_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char('}')(s)
        }
    }

    fn parse_tilde_char(&self) -> impl Fn(&str) -> IResult<&str, char> + '_  {
        move |s| {
            self.single_char('~')(s)
        }
    }
}

/* ---------- strings ----------*/

impl Parser {

    fn parse_space_string(&self) -> impl Fn(&str) -> IResult<&str, String> + '_ {
        move |s|{
            map(self.parse_space(),|input_s: char| input_s.to_string() )(s)
        }
    }
    
    fn parse_line_break(&self) -> impl Fn(&str) -> IResult<&str, &str> + '_ {
        move |s| {
            match line_ending(s) {
                Ok(r) => {

                    /* for debug */
                    let depth = self.pos_tran_depth();
                    debug!("{empty:>width$}POS({}): INCREASE LINE {:?}", depth, r, width=depth*2, empty="");

                    self.increase_line();
                    Ok(r)
                },
                Err(e) => Err(e)
            }
        }
    }

    // ソフトブレイク
    fn parse_soft_break(&self) -> impl Fn(&str) -> IResult<&str, String> + '_  {
        move |s| {
            map(self.parse_line_break(), |input_s: &str| input_s.to_owned())(s)
        }
    }

    fn parse_soft_break_node(&self) -> impl Fn(&str) -> IResult<&str, ASTNode> + '_  {
        move |s| {
            match self.parse_soft_break()(s) {
                Ok((remain, _)) => {
                    let node = ASTNode::new(ASTElm::new_softbreak( self.pos_get_range() ));
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

    fn parse_hard_break_node(&self) -> impl Fn(&str) -> IResult<&str, ASTNode> + '_  {
        move |s| {
            match self.parse_hard_break()(s) {
                Ok((remain, _)) => {
                    let node = ASTNode::new(ASTElm::new_hardbreak( self.pos_get_range() ));
                    Ok((remain, node))
                }
                Err(e) => Err(e),
            }
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
            match alt(with_tran!(self,
                    self.parse_inline_syntax(),
                    self.parse_sp_symbol()
            ))(s)
            {
                Ok((remain, node)) => Ok((remain, node)),
                Err(_) => Err(Err::Error(ParseError::from_error_kind(s, ErrorKind::Char))),
            }
        }
    }

    fn parse_inline_syntax(&self) -> impl Fn(&str) -> IResult<&str, ASTNode> + '_  {
        move |s| {
            match alt(with_tran!(self,
                    map(self.parse_nsp_string(), |input_s: String| {
                        let node = ASTNode::new( ASTElm::new_text( &input_s, self.pos_get_range() ));
                        return node;
                    }),
                    self.parse_emphasis()
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
     * NOTE: これはLISPですか？;D
     * TODO: アスタリスク前後の記号の取り扱いをcommonmarkに合わせる
     * */
    fn parse_emphasis(&self) -> impl Fn(&str) -> IResult<&str, ASTNode> + '_  {
        move |s| {
            match with_tran!(self,
                delimited(
                    tuple((self.single_char('*'), isolate!(self, peek(not(alt((self.parse_soft_break(),self.parse_space_string()))))))),
                    many1(alt(with_tran!(self, self.parse_inline_syntax(), self.parse_soft_break_node()))),
                    tuple((isolate!(self, peek(not(alt((self.parse_soft_break(),self.parse_space_string()))))), self.single_char('*'))),
                )
            )(s)
            {
                Ok((remain, nodes)) => {
                    let mut node = ASTNode::new(ASTElm::new_emphasis(
                            "", s.slice(..s.len()-remain.len()), self.pos_get_range()
                            ));
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
                let node = ASTNode::new(ASTElm::new_text( &input_s.to_string(), self.pos_get_range() ));
                return node;
            })(s)
            {
                Ok((remain, node)) => Ok((remain, node)),
                Err(_) => Err(Err::Error(ParseError::from_error_kind(s, ErrorKind::Char))),
            }
        }
    }

    /*
     * 構文上意味がない漏れてきた記号のパース
     */
    fn parse_sp_symbol(&self) -> impl Fn(&str) -> IResult<&str, ASTNode> + '_ {
        move |s| {
            match map(self.parse_sp_char(), |input_s: char| {
                let node = ASTNode::new(ASTElm::new_text( &input_s.to_string(), self.pos_get_range() ));
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
            let r = with_tran!(self, tuple((
                    many_m_n(0, 3, self.parse_space()),
                    map(many_m_n(1, 6, self.single_char('#')), |input_s: Vec<char>| {
                        input_s.len()
                    }),
                    //space1,
                    many1(self.single_char(' ')),
                    map(isolate!(self, self.parse_nbr_string()), |input_s: String| {
                        many1(self.parse_inline())(&input_s).unwrap().1
                    }),
                    many0((self.single_char('#'))),
                    isolate!(self, peek(self.parse_break_or_eof())),
            )))(s);
            match r {
                Ok((remain, (_, level, _, inline_node, _, _))) => {
                    let mut node = ASTNode::new( ASTElm::new_headers(
                            level,
                            "", s.slice(..s.len()-remain.len()),
                            self.pos_get_previous_line_range()
                    ));
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
            match map(isolate!(self, self.parse_separate()), |input_s: String| {
                ( input_s.to_owned(), many1(alt((
                        self.parse_inline(),
                        self.parse_soft_break_node(),
                        self.parse_sp_asterisk(),
                )))(&input_s).unwrap().1 )
            })(s) {
                Ok((remain, (raw_value, child_node))) => {
                    let mut node = ASTNode::new(ASTElm::new_paragraph(
                            "", &raw_value, self.pos_get_range()
                    ));
                    node.append_node_from_vec(child_node);
                    Ok((remain, node))
                }
                Err(_) => Err(Err::Error(ParseError::from_error_kind(s, ErrorKind::Char))),
            }
        }
    }

    // 入力位置からブロック終了までの文字を返す
    fn parse_separate(&self) -> impl Fn(&str) -> IResult<&str, String> + '_ {
        move |s| {
            match map(
                many1(
                    map(tuple((
                            self.parse_nbr_string(),
                            self.parse_separator(),
                    )),|(a,b): (String, String)| {
                        a + &b
                    })
                ), |input_s: Vec<String>| {
                    util_vecstring_to_string(input_s)
                }
            )(s) {
                Ok((remain, text)) => {
                    Ok((remain, text))
                },
                Err(e) => Err(e)
            }
        }
    }

    // ブロック終了の判定
    // 終了判断: Headerの書式、ハードブレイク
    fn parse_separator(&self) -> impl Fn(&str) -> IResult<&str, String> + '_  {
        move |s| {
            let r = tuple((
                    peek(not(self.parse_hard_break())),
                    peek(not(tuple(( self.parse_break_or_eof(), not(self.parse_nbr_string()),)))),
                    self.parse_break_or_eof(),
            ))(s);
            match r {
                Ok((remain, (_, _, br))) => {

                    // 他のブロックを判定
                    match tuple((
                            // headering
                            peek(not(tuple((
                                    many_m_n(0, 3, self.parse_space()),
                                    map(many_m_n(1, 6, self.single_char('#')), |input_s: Vec<char>| {
                                        input_s.len()
                                    }),
                                    many1(self.single_char(' ')),
                            )))),
                    ))(remain) {
                        Ok(_) => Ok((remain, br)),
                        Err(_) => Ok((s, "".to_string())),
                    }

                },
                Err(_) => Ok((s, "".to_string())),
            }
        }
    }

    fn parse_blocks(&self) -> impl Fn(&str) -> IResult<&str, ASTNode> + '_  {
        move |s| {
            alt( with_tran!(self,
                    self.parse_headers(),
                    self.parse_paragraph(),
                    
                    // 改行を無視する
                    map( tuple((self.parse_line_break(), self.parse_blocks()) ), |(_, node)| node)
            ))(s)
        }
    }

    fn parse_document(&self, s: &str, mut node: ASTNode) -> ASTNode {
        node.set_node_type(ASTType::Document);
        node.set_meta(ASTMetaData::Nil);
        node.set_value("".to_string());
        node.set_raw_value(s.to_string());

        match with_tran!(self, many0(with_tran!(self, self.parse_blocks())))(s) {
            Ok((remain, result)) => {
                node.append_node_from_vec(result);
                node.set_range( self.pos_get_range() );
            }
            Err(_) => {}
        }

        return node;
    }
}

pub fn md_parse(s: &str, node: ASTNode) -> ASTNode {
    let parser = Parser::new();
    parser.parse_document(s, node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::Err;
    use nom::error::Error;

    #[test]
    fn test_single_char_ok(){
        let parser = Parser::new();
        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(1,1,0)));
        assert_eq!(parser.single_char('a')("abc"), Ok(("bc", 'a')));
        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(1,2,1))); 
    }
    
    #[test]
    fn test_single_char_ok_isolate(){
        let parser = Parser::new();
        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(1,1,0)));
        assert_eq!(isolate!(parser, parser.single_char('a'))("abc"), Ok(("bc", 'a')));
        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(1,1,0))); 
    }

    #[test]
    fn test_single_char_err(){
        let parser = Parser::new();
        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(1,1,0)));
        assert_eq!(parser.single_char('b')("abc"), Err(Err::Error(Error::new("abc", ErrorKind::Char ))));
        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(1,1,0))); 
    }

    #[test]
    fn test_parse_emphasis_ok(){
        let parser = Parser::new();
        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(1,1,0)));
        assert_eq!(parser.parse_emphasis()("*emphasis*").unwrap().1.render_debug_format(), "<emphasis><text>emphasis</text></emphasis>");
        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(1,11,10)));
    }
    
    #[test]
    fn test_parse_emphasis_multiline_ok(){
        let parser = Parser::new();
        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(1,1,0)));

        assert_eq!(parser.parse_emphasis()("*emphasis\nemphasis*").unwrap().1.render_debug_format(),
        "<emphasis><text>emphasis</text><softbreak /><text>emphasis</text></emphasis>");

        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(2,10,19)));
    }

    #[test]
    fn test_parse_emphasis_nested_ok(){
        let parser = Parser::new();
        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(1,1,0)));

        assert_eq!(parser.parse_emphasis()("*emphasis\n*emphasis*\nemphasis*").unwrap().1.render_debug_format(),
        "<emphasis><text>emphasis</text><softbreak /><emphasis><text>emphasis</text></emphasis><softbreak /><text>emphasis</text></emphasis>");

        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(3,10,30)));
    }

    #[test]
    fn test_parse_emphasis_err(){
        let parser = Parser::new();
        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(1,1,0)));

        assert_eq!(parser.parse_emphasis()("*invalid\nemphasis"),
        Err(Err::Error(Error::new("*invalid\nemphasis", ErrorKind::Char ))));

        assert_eq!(parser.parse_emphasis()("invalid\nemphasis*"),
        Err(Err::Error(Error::new("invalid\nemphasis*", ErrorKind::Char ))));

        assert_eq!(parser.parse_emphasis()("*invalid\n*emphasis*"),
        Err(Err::Error(Error::new("*invalid\n*emphasis*", ErrorKind::Char ))));

        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(1,1,0)));
    }

    #[test]
    fn test_parse_inline_ok(){
        let parser = Parser::new();
        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(1,1,0)));

        assert_eq!(parser.parse_inline()("this is text*this is Emphasis*"),
        Ok((
                "*this is Emphasis*",
                ASTNode::new( 
                    ASTElm::new_text("this is text",
                        ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(1,13,12) )
                    )
                )
        )
        )
        );

        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(1,13,12)));
    }

    #[test]
    fn test_parse_paragraph(){
        let parser = Parser::new();
        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(1,1,0)));

        assert_eq!(parser.parse_paragraph()("this is text*this is Emphasis*").unwrap().1.render_debug_format(),
        "<paragraph><text>this is text</text><emphasis><text>this is Emphasis</text></emphasis></paragraph>"
        );

        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(1,31,30)));
    }

    #[test]
    fn test_parse_paragraph_multiline(){
        let parser = Parser::new();
        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(1,1,0)));

        assert_eq!(parser.parse_paragraph()("this is text\nmultiline").unwrap().1.render_debug_format(),
        "<paragraph><text>this is text</text><softbreak /><text>multiline</text></paragraph>"
        );

        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(2,10,22)));
    }

    #[test]
    fn test_parse_paragraph_multiline_other_block(){
        let parser = Parser::new();
        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(1,1,0)));

        assert_eq!(parser.parse_paragraph()("this is text\nmultiline\n# headering").unwrap().1.render_debug_format(),
        "<paragraph><text>this is text</text><softbreak /><text>multiline</text></paragraph>"
        );

        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(2,10,22)));

        let parser = Parser::new();
        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(1,1,0)));
        assert_eq!(parser.parse_paragraph()("this is text\nmultiline\n# headering").unwrap().0, "\n# headering");
        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(2,10,22)));
    }

    #[test]
    fn test_parse_headers(){
        let parser = Parser::new();
        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(1,1,0)));

        assert_eq!(parser.parse_headers()("# *headering*").unwrap().1.render_debug_format(),
        "<header><emphasis><text>headering</text></emphasis></header>");

        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(1,14,13)));
    }

    #[test]
    fn test_parse_document(){
        let parser = Parser::new();
        let mut node = ASTNode::new( ASTElm::new_document() );
        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(1,1,0)));

        node = parser.parse_document("# *headering*\n\nthis is paragraph\n*this is emphasis*\n\nthis is other paragraph", node);

        assert_eq!(parser.pos_get_range(), ASTRange::new( ASTPos::new(1,1,0), ASTPos::new(6,24,76)));

        assert_eq!(node.render_debug_format(),
        "<document><header><emphasis><text>headering</text></emphasis></header><paragraph><text>this is paragraph</text><softbreak /><emphasis><text>this is emphasis</text></emphasis></paragraph><paragraph><text>this is other paragraph</text></paragraph></document>"
        );

    }
}
