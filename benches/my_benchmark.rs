#[macro_use]
extern crate criterion;

use orgmd_parser::module::ast::{
    ASTNode, ASTElm, 
};
use orgmd_parser::module::md_parser::md_parse;

use criterion::Criterion;
use criterion::black_box;

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}

fn parse_markdown(value: &str) -> ASTNode {
    let mut node = ASTNode::new( ASTElm { ..Default::default() } );
    md_parse(value, node)
}

fn n_parse_markdown(n: u64) -> ASTNode {
    let mut value = "".to_string();
    for i in 1..n {
        value = value + "# big markdown\n*emphasis*\nparagraph\n\n";
    }
    parse_markdown(&value)
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
    c.bench_function("markdown 100", |b| b.iter(|| n_parse_markdown(black_box(100))));
    c.bench_function("markdown 500", |b| b.iter(|| n_parse_markdown(black_box(500))));
    c.bench_function("markdown 1000", |b| b.iter(|| n_parse_markdown(black_box(1000))));
    c.bench_function("markdown 10000", |b| b.iter(|| n_parse_markdown(black_box(10000))));
    c.bench_function("markdown 100000", |b| b.iter(|| n_parse_markdown(black_box(100000))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
