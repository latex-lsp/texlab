use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lsp_types::{Position, Url};
use texlab::{
    db::{Language, Owner, Workspace},
    parser::parse_latex,
    Database,
};

const CODE: &str = include_str!("../../../texlab.tex");

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("LaTeX/Parser", |b| {
        b.iter(|| parse_latex(black_box(CODE)));
    });

    c.bench_function("LaTeX/Completion/Command", |b| {
        let uri = Url::parse("http://example.com/texlab.tex").unwrap();
        let text = CODE.to_string();
        let mut db = Database::default();
        Workspace::get(&db).open(&mut db, uri.clone(), text, Language::Tex, Owner::Client);
        b.iter(|| texlab::features::completion::complete(&db, &uri, Position::new(0, 1)));
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
