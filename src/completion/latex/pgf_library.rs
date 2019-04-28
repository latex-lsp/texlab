use crate::completion::factory;
use crate::completion::latex::combinators::LatexCombinators;
use crate::feature::FeatureRequest;
use lsp_types::{CompletionItem, CompletionParams};

pub struct LatexPgfLibraryCompletionProvider;

impl LatexPgfLibraryCompletionProvider {
    pub async fn execute(request: &FeatureRequest<CompletionParams>) -> Vec<CompletionItem> {
        await!(LatexCombinators::argument(request, &COMMANDS, 0, async move |_| {
            LIBRARIES.iter()
                .map(|name| factory::create_pgf_library((*name).to_owned()))
                .collect()
        }))
    }
}

const COMMANDS: &'static [&'static str] = &["\\usepgflibrary"];

const LIBRARIES: &'static [&'static str] = &[
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
    use crate::feature::FeatureTester;
    use crate::workspace::WorkspaceBuilder;
    use futures::executor;

    #[test]
    fn test() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "\\usepgflibrary{}");
        let request = FeatureTester::new(builder.workspace, uri, 0, 15, "").into();

        let items = executor::block_on(LatexPgfLibraryCompletionProvider::execute(&request));

        assert_eq!(true, items.iter().any(|item| item.label == "arrows"));
    }
}
