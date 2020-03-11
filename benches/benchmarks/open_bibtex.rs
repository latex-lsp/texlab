use criterion::Criterion;
use std::time::Duration;
use texlab::{syntax::bibtex, tex::Distribution};
use tokio::fs;

async fn criterion_benchmark(criterion: &mut Criterion) {
    let distro = Distribution::detect().await;
    distro
        .load()
        .await
        .expect("failed to load TeX distribution");
    let resolver = distro.resolver().await;
    let path = resolver
        .files_by_name
        .get("biblatex-examples.bib")
        .expect("unable to retrieve biblatex-examples.bib");

    let text = fs::read_to_string(&path).await.unwrap();
    criterion.bench_function("biblatex-examples.bib", |b| b.iter(|| bibtex::open(&text)));
}

pub async fn benches() {
    let mut criterion = Criterion::default()
        .measurement_time(Duration::from_secs(20))
        .configure_from_args();
    criterion_benchmark(&mut criterion).await;
}
