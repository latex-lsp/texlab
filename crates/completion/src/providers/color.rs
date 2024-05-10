use rowan::ast::AstNode;
use syntax::latex;

use crate::{
    util::{find_curly_group_word, CompletionBuilder},
    CompletionItem, CompletionItemData, CompletionParams,
};

pub fn complete_colors<'a>(
    params: &'a CompletionParams<'a>,
    builder: &mut CompletionBuilder<'a>,
) -> Option<()> {
    let (cursor, group) = find_curly_group_word(params)?;
    latex::ColorReference::cast(group.syntax().parent()?)?;

    for name in COLORS {
        if let Some(score) = builder.matcher.score(name, &cursor.text) {
            let data = CompletionItemData::Color(name);
            builder
                .items
                .push(CompletionItem::new_simple(score, cursor.range, data));
        }
    }

    Some(())
}

const COLORS: &[&str] = &[
    "black",
    "blue",
    "brown",
    "cyan",
    "darkgray",
    "gray",
    "green",
    "lightgray",
    "lime",
    "magenta",
    "olive",
    "orange",
    "pink",
    "purple",
    "red",
    "teal",
    "violet",
    "white",
    "yellow",
    "Apricot",
    "Bittersweet",
    "Blue",
    "BlueViolet",
    "Brown",
    "CadetBlue",
    "Cerulean",
    "Cyan",
    "DarkOrchid",
    "ForestGreen",
    "Goldenrod",
    "Green",
    "JungleGreen",
    "LimeGreen",
    "Mahogany",
    "Melon",
    "Mulberry",
    "OliveGreen",
    "OrangeRed",
    "Peach",
    "PineGreen",
    "ProcessBlue",
    "RawSienna",
    "RedOrange",
    "Rhodamine",
    "RoyalPurple",
    "Salmon",
    "Sepia",
    "SpringGreen",
    "TealBlue",
    "Turquoise",
    "VioletRed",
    "WildStrawberry",
    "YellowGreen",
    "Aquamarine",
    "Black",
    "BlueGreen",
    "BrickRed",
    "BurntOrange",
    "CarnationPink",
    "CornflowerBlue",
    "Dandelion",
    "Emerald",
    "Fuchsia",
    "Gray",
    "GreenYellow",
    "Lavender",
    "Magenta",
    "Maroon",
    "MidnightBlue",
    "NavyBlue",
    "Orange",
    "Orchid",
    "Periwinkle",
    "Plum",
    "Purple",
    "Red",
    "RedViolet",
    "RoyalBlue",
    "RubineRed",
    "SeaGreen",
    "SkyBlue",
    "Tan",
    "Thistle",
    "Violet",
    "White",
    "Yellow",
    "YellowOrange",
];
