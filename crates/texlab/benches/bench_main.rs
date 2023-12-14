use criterion::{black_box, criterion_group, criterion_main, Criterion};
use parser::{parse_latex, SyntaxConfig};

const CODE: &str = include_str!("../../../texlab.tex");

fn criterion_benchmark(c: &mut Criterion) {
    let config = SyntaxConfig::default();
    c.bench_function("LaTeX/Parser", |b| {
        b.iter(|| parse_latex(black_box(CODE), &config));
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
