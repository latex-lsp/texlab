use crate::completion::factory;
use crate::completion::latex::combinators::LatexCombinators;
use crate::feature::FeatureRequest;
use lsp_types::{CompletionItem, CompletionParams};
use std::borrow::Cow;

pub struct LatexPgfLibraryCompletionProvider;

impl LatexPgfLibraryCompletionProvider {
    pub async fn execute(request: &FeatureRequest<CompletionParams>) -> Vec<CompletionItem> {
        await!(LatexCombinators::argument(
            request,
            &COMMANDS,
            0,
            async move |_| {
                LIBRARIES
                    .iter()
                    .map(|name| factory::create_pgf_library(Cow::from(*name)))
                    .collect()
            }
        ))
    }
}

const COMMANDS: &[&str] = &["\\usepgflibrary"];

const LIBRARIES: &[&str] = &[
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
    use crate::feature::FeatureSpec;
    use crate::test_feature;
    use lsp_types::Position;

    #[test]
    fn test() {
        let items = test_feature!(
            LatexPgfLibraryCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\usepgflibrary{}")],
                main_file: "foo.tex",
                position: Position::new(0, 15),
                ..FeatureSpec::default()
            }
        );
        assert_eq!(items.iter().any(|item| item.label == "arrows"), true);
    }
}
