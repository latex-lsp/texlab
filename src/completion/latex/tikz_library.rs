use crate::completion::factory;
use crate::completion::latex::combinators::LatexCombinators;
use crate::feature::FeatureRequest;
use lsp_types::{CompletionItem, CompletionParams};

pub struct LatexTikzLibraryCompletionProvider;

impl LatexTikzLibraryCompletionProvider {
    pub async fn execute(request: &FeatureRequest<CompletionParams>) -> Vec<CompletionItem> {
        await!(LatexCombinators::argument(
            request,
            &COMMANDS,
            0,
            async move |_| {
                LIBRARIES
                    .iter()
                    .map(|name| factory::create_pgf_library((*name).to_owned()))
                    .collect()
            }
        ))
    }
}

const COMMANDS: &'static [&'static str] = &["\\usetikzlibrary"];

const LIBRARIES: &'static [&'static str] = &[
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
    use crate::feature::FeatureTester;
    use crate::workspace::WorkspaceBuilder;
    use futures::executor::block_on;

    #[test]
    fn test() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "\\usetikzlibrary{}");
        let request = FeatureTester::new(builder.workspace, uri, 0, 16, "").into();

        let items = block_on(LatexTikzLibraryCompletionProvider::execute(&request));

        assert_eq!(true, items.iter().any(|item| item.label == "arrows"));
    }
}
