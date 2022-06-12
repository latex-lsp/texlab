use std::sync::Arc;

use glassbench::{glassbench, pretend_used, Bench};
use lsp_types::{CompletionParams, Url};
use texlab::{features::FeatureRequest, syntax::latex, DocumentLanguage, Workspace};

fn bench_parser(bench: &mut Bench) {
    bench.task("Parser/LaTeX", |task| {
        task.iter(|| pretend_used(latex::parse(include_str!("../texlab.tex"))));
    });
}

fn bench_completion(bench: &mut Bench) {
    bench.task("Completion/LaTeX/Command", |task| {
        let uri = Arc::new(Url::parse("http://example.com/texlab.tex").unwrap());
        let text = Arc::new(include_str!("../texlab.tex").to_string());
        let params: CompletionParams = serde_json::from_value(serde_json::json!({
            "textDocument": {
                "uri": uri.as_str()
            },
            "position": {
                "line": 0u32,
                "character": 1u32
            }
        }))
        .unwrap();

        task.iter(|| {
            let mut workspace = Workspace::default();
            workspace
                .open(Arc::clone(&uri), Arc::clone(&text), DocumentLanguage::Latex)
                .unwrap();

            pretend_used(texlab::features::complete(FeatureRequest {
                params: params.clone(),
                workspace: workspace.clone(),
                uri: Arc::clone(&uri),
            }))
        });
    });
}

glassbench!("texlab", bench_parser, bench_completion,);
