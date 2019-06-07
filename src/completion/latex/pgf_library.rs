use crate::completion::factory;
use crate::completion::latex::combinators::LatexCombinators;
use crate::feature::{FeatureProvider, FeatureRequest};
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams};
use std::borrow::Cow;
use std::sync::Arc;

pub struct LatexPgfLibraryCompletionProvider {
    items: Vec<Arc<CompletionItem>>,
}

impl LatexPgfLibraryCompletionProvider {
    pub fn new() -> Self {
        let items = LIBRARIES
            .iter()
            .map(|name| Cow::from(*name))
            .map(factory::create_pgf_library)
            .map(Arc::new)
            .collect();
        Self { items }
    }
}

impl FeatureProvider for LatexPgfLibraryCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<Arc<CompletionItem>>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        LatexCombinators::argument(request, &COMMANDS, 0, async move |_| self.items.clone()).await
    }
}

const COMMANDS: &[&str] = &["\\usepgflibrary"];

static LIBRARIES: &[&str] = &[
    "arrows",
    "arrows.meta",
    "arrows.spaced",
    "curvilinear",
    "datavisualization.barcharts",
    "datavisualization.formats.functions",
    "datavisualization.polar",
    "decorations.footprints",
    "decorations.fractals",
    "decorations.markings",
    "decorations.pathmorphing",
    "decorations.pathreplacing",
    "decorations.shapes",
    "decorations.text",
    "fadings",
    "fixedpointarithmetic",
    "fpu",
    "intersections",
    "lindenmayersystems",
    "luamath",
    "patterns",
    "patterns.meta",
    "plothandlers",
    "plotmarks",
    "profiler",
    "shadings",
    "shapes.arrows",
    "shapes.callouts",
    "shapes",
    "shapes.gates.ee",
    "shapes.gates.ee.IEC",
    "shapes.gates.logic",
    "shapes.gates.logic.IEC",
    "shapes.gates.logic.US",
    "shapes.geometric",
    "shapes.misc",
    "shapes.multipart",
    "shapes.symbols",
    "snakes",
    "svg.path",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::{test_feature, FeatureSpec};
    use lsp_types::Position;

    #[test]
    fn test() {
        let items = test_feature(
            LatexPgfLibraryCompletionProvider::new(),
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\usepgflibrary{}")],
                main_file: "foo.tex",
                position: Position::new(0, 15),
                ..FeatureSpec::default()
            },
        );
        assert!(items.iter().any(|item| item.label == "arrows"));
    }
}
