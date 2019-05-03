use crate::completion::factory;
use crate::completion::latex::combinators::LatexCombinators;
use crate::feature::FeatureRequest;
use lsp_types::{CompletionItem, CompletionParams};

pub struct LatexColorCompletionProvider;

impl LatexColorCompletionProvider {
    pub async fn execute(request: &FeatureRequest<CompletionParams>) -> Vec<CompletionItem> {
        await!(LatexCombinators::argument(
            request,
            &COLOR_COMMANDS,
            0,
            async move |_| {
                COLOR_NAMES
                    .iter()
                    .map(|name| factory::create_color((*name).to_owned()))
                    .collect()
            }
        ))
    }
}

const COLOR_COMMANDS: &'static [&'static str] = &[
    "\\color",
    "\\colorbox",
    "\\textcolor",
    "\\pagecolor",
    "\\colorlet",
    "\\definespotcolor",
];

const COLOR_NAMES: &'static [&'static str] = &[
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::completion::latex::data::types::LatexComponentDatabase;
    use crate::feature::FeatureSpec;
    use crate::test_feature;
    use lsp_types::Position;

    #[test]
    fn test_inside_color() {
        let items = test_feature!(
            LatexColorCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\color{}")],
                main_file: "foo.tex",
                position: Position::new(0, 7),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(true, items.iter().any(|item| item.label == "black"));
    }

    #[test]
    fn test_outside_color() {
        let items = test_feature!(
            LatexColorCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\color{}")],
                main_file: "foo.tex",
                position: Position::new(0, 8),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(items, Vec::new());
    }
}
