use rowan::ast::AstNode;
use syntax::latex;

use crate::{
    CompletionItem, CompletionItemData, CompletionParams,
    util::{CompletionBuilder, find_curly_group_word_list},
};

pub fn complete_tikz_libraries<'a>(
    params: &'a CompletionParams,
    builder: &mut CompletionBuilder<'a>,
) -> Option<()> {
    let (cursor, group) = find_curly_group_word_list(params)?;

    let import = latex::TikzLibraryImport::cast(group.syntax().parent()?)?;

    let libraries = if import.command()?.text() == "\\usepgflibrary" {
        PGF_LIBRARIES
    } else {
        TIKZ_LIBRARIES
    };

    for name in libraries {
        if let Some(score) = builder.matcher.score(name, &cursor.text) {
            let data = CompletionItemData::TikzLibrary(name);
            builder
                .items
                .push(CompletionItem::new_simple(score, cursor.range, data));
        }
    }

    Some(())
}

static PGF_LIBRARIES: &[&str] = &[
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

static TIKZ_LIBRARIES: &[&str] = &[
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
