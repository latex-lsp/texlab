use crate::completion::factory;
use crate::completion::latex::combinators::{self, ArgumentLocation};
use crate::feature::{FeatureProvider, FeatureRequest};
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams};
use std::borrow::Cow;
use std::sync::Arc;

pub struct LatexTikzLibraryCompletionProvider {
    items: Vec<Arc<CompletionItem>>,
}

impl LatexTikzLibraryCompletionProvider {
    pub fn new() -> Self {
        let items = LIBRARIES
            .iter()
            .map(|name| Cow::from(*name))
            .map(factory::create_tikz_library)
            .map(Arc::new)
            .collect();
        Self { items }
    }
}

impl FeatureProvider for LatexTikzLibraryCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<Arc<CompletionItem>>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let locations = COMMANDS.iter().map(|cmd| ArgumentLocation::new(cmd, 0));
        combinators::argument(request, locations, async move |_| self.items.clone()).await
    }
}

const COMMANDS: &[&str] = &["\\usetikzlibrary"];

const LIBRARIES: &[&str] = &[
    "3d",
    "angles",
    "arrows",
    "automata",
    "babel",
    "backgrounds",
    "bending",
    "calc",
    "calendar",
    "chains",
    "circuits",
    "circuits.ee",
    "circuits.ee.IEC",
    "circuits.logic.CDH",
    "circuits.logic",
    "circuits.logic.IEC",
    "circuits.logic.US",
    "datavisualization.3d",
    "datavisualization.barcharts",
    "datavisualization",
    "datavisualization.formats.functions",
    "datavisualization.polar",
    "datavisualization.sparklines",
    "decorations",
    "decorations.footprints",
    "decorations.fractals",
    "decorations.markings",
    "decorations.pathmorphing",
    "decorations.pathreplacing",
    "decorations.shapes",
    "decorations.text",
    "er",
    "fadings",
    "fit",
    "fixedpointarithmetic",
    "folding",
    "fpu",
    "graphs",
    "graphs.standard",
    "intersections",
    "lindenmayersystems",
    "math",
    "matrix",
    "mindmap",
    "patterns",
    "patterns.meta",
    "petri",
    "plothandlers",
    "plotmarks",
    "positioning",
    "quotes",
    "scopes",
    "shadings",
    "shadows",
    "shapes.arrows",
    "shapes.callouts",
    "shapes",
    "shapes.gates.logic.IEC",
    "shapes.gates.logic.US",
    "shapes.geometric",
    "shapes.misc",
    "shapes.multipart",
    "shapes.symbols",
    "snakes",
    "spy",
    "svg.path",
    "through",
    "topaths",
    "trees",
    "turtle",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::{test_feature, FeatureSpec};
    use lsp_types::Position;

    #[test]
    fn test() {
        let items = test_feature(
            LatexTikzLibraryCompletionProvider::new(),
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\usetikzlibrary{}")],
                main_file: "foo.tex",
                position: Position::new(0, 16),
                ..FeatureSpec::default()
            },
        );
        assert!(items.iter().any(|item| item.label == "arrows"));
    }
}
