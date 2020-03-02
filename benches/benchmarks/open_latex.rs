use criterion::Criterion;
use std::{env, time::Duration};
use texlab::{
    protocol::{Options, Uri},
    syntax::latex,
    tex::Distribution,
};
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
        .get("symbols.tex")
        .expect("unable to retrieve symbols.tex");

    let text = fs::read_to_string(&path).await.unwrap();
    let uri = Uri::from_file_path(&path).unwrap();
    let options = Options::default();
    let cwd = env::current_dir().unwrap();
    let params = latex::OpenParams {
        text: &text,
        uri: &uri,
        resolver: &resolver,
        options: &options,
        cwd: &cwd,
    };

    criterion.bench_function("symbols.tex", |b| b.iter(|| latex::open(params)));
}

pub async fn benches() {
    let mut criterion = Criterion::default()
        .configure_from_args()
        .sample_size(15)
        .measurement_time(Duration::from_secs(30));
    criterion_benchmark(&mut criterion).await;
}
