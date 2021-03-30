use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};
use texlab::{
    db::*,
    protocol::Uri,
    syntax::{bibtex, latex},
    DocumentLanguage,
};

static TEX_CODE: &str = include_str!("D:/uni/bachelor-thesis/ausarbeitung/thesis.tex");
static BIB_CODE: &str = include_str!("bench.bib");

fn analysis(criterion: &mut Criterion) {
    let tex_uri = Arc::new(Uri::parse("http://www.example.com/foo.tex").unwrap());
    let tex_code = Arc::new(TEX_CODE.to_string());

    criterion.bench_function("LaTeX/Parser", |b| b.iter(|| latex::parse::<()>(TEX_CODE)));

    criterion.bench_function("BibTeX/Parser", |b| {
        b.iter(|| bibtex::parse::<()>(BIB_CODE))
    });

    criterion.bench_function("LaTeX/Analysis", |b| {
        b.iter_with_setup(
            || {
                let mut db = RootDatabase::default();
                let document = db.intern_document(DocumentData {
                    uri: Arc::clone(&tex_uri),
                });
                db.set_text(document, Arc::clone(&tex_code));
                db.set_language(document, DocumentLanguage::Latex);
                (db, document)
            },
            |(db, document)| (db.tree(document), db),
        )
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = analysis
}

criterion_main! { benches }
