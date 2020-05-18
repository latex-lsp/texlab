use super::combinators::{self, Parameter};
use crate::{
    completion::types::{Item, ItemData},
    feature::FeatureRequest,
    protocol::CompletionParams,
    syntax::{
        LatexGlossaryEntryKind::{Acronym, General},
        LANGUAGE_DATA,
    },
    workspace::DocumentContent,
};

pub async fn complete_latex_glossary_entries<'a>(
    req: &'a FeatureRequest<CompletionParams>,
    items: &mut Vec<Item<'a>>,
) {
    let parameters = LANGUAGE_DATA
        .glossary_entry_reference_commands
        .iter()
        .map(|cmd| Parameter {
            name: &cmd.name[1..],
            index: cmd.index,
        });

    combinators::argument(req, parameters, |ctx| async move {
        let cmd_kind = LANGUAGE_DATA
            .glossary_entry_reference_commands
            .iter()
            .find(|cmd| &cmd.name[1..] == ctx.parameter.name)
            .unwrap()
            .kind;

        for doc in req.related() {
            if let DocumentContent::Latex(table) = &doc.content {
                for entry in &table.glossary_entries {
                    match (cmd_kind, entry.kind) {
                        (Acronym, Acronym) | (General, General) | (General, Acronym) => {
                            let name = entry.label(&table).text();
                            let item = Item::new(ctx.range, ItemData::GlossaryEntry { name });
                            items.push(item);
                        }
                        (Acronym, General) => {}
                    }
                }
            }
        }
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        feature::FeatureTester,
        protocol::{Range, RangeExt},
    };
    use indoc::indoc;

    #[tokio::test]
    async fn empty_latex_document() {
        let req = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_glossary_entries(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let req = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_glossary_entries(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn acronym() {
        let req = FeatureTester::new()
            .file(
                "main.tex",
                indoc!(
                    r#"
                        \newacronym{lvm}{LVM}{Logical Volume Manager}
                        \acrfull{foo}
                    "#
                ),
            )
            .main("main.tex")
            .position(1, 9)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_glossary_entries(&req, &mut actual_items).await;

        assert_eq!(actual_items.len(), 1);
        assert_eq!(actual_items[0].data.label(), "lvm");
        assert_eq!(actual_items[0].range, Range::new_simple(1, 9, 1, 12));
    }
}
