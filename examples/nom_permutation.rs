use nom::IResult;
use nom::branch::permutation;
use nom::character::complete::{anychar, char};

fn parser(s: &str) -> IResult<&str, (char, char)>{
    permutation((anychar, char('a')))(s)
}

fn main(){
    println!("{:?}", parser("ab"));
}
