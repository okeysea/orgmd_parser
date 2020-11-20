extern crate nom;
use nom::{
    Err, IResult, InputIter, Slice, AsChar,
    InputTakeAtPosition, InputLength,
};
use nom::error::{
    ErrorKind, ParseError,
};
use nom::character::complete::{
    char, digit1, multispace0, line_ending,
    not_line_ending,
};
use nom::branch::alt;
use nom::sequence::delimited;
use nom::multi::many_m_n;
use std::ops::RangeFrom;

/*
fn numeric_in_parantheses(s: &str) -> IResult<&strm &str> {
    let (s, _) = char('(')(s)?;
    let (s, _) = multispace0(s)?;
    let (s, num) = digit1(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = char(')')(s)?;
    Ok((s, num))
}*/

fn decide_char_printable(input: char) -> Result<char, String>
{
    let i = input as u32;
    let mut r = false;
    
    // 制御文字の判定
    r = 0x00 >= i  && i <= 0x1F;
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
    
fn printable_char_test(s: &str) -> IResult<&str, char> {
    printable_char('a')(s)
}

fn printable_char_test3(s: &str) -> IResult<&str, char> {
    printable_char3(s)
}
// non-break char
fn nbr_char(s: &str) -> IResult<&str, &str> {
    not_line_ending(s)
}

fn soft_break(s: &str) -> IResult<&str, &str> {
    line_ending(s)
}

fn hard_break(s: &str) -> IResult<&str, Vec<&str>> {
    many_m_n(2, 99, soft_break)(s)
}

fn parse_char(s: &str) -> IResult<&str, &str> {
    alt((
        nbr_char,
        soft_break,
    ))(s)
}

fn numeric_in_parantheses(s: &str) -> IResult<&str, &str> {
    delimited(
        char('('),
        delimited(multispace0, digit1, multispace0),
        char(')'),
    )(s)
}

fn blocks(s: &str) -> IResult<&str, &str> {
    alt((
        headers,
        paragraph
    ))(s)
}
