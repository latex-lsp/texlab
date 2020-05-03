use super::combinators::{self, Parameter};
use crate::factory;
use async_trait::async_trait;
use texlab_feature::{DocumentContent, FeatureProvider, FeatureRequest};
use texlab_protocol::{CompletionItem, CompletionParams, TextEdit};
use texlab_syntax::{
    LatexGlossaryEntryKind::{Acronym, General},
    LANGUAGE_DATA,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexGlossaryCompletionProvider;

#[async_trait]
impl FeatureProvider for LatexGlossaryCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let parameters = LANGUAGE_DATA
            .glossary_entry_reference_commands
            .iter()
            .map(|cmd| Parameter {
                name: &cmd.name,
                index: cmd.index,
            });

        combinators::argument(req, parameters, |ctx| async move {
            let cmd_kind = LANGUAGE_DATA
                .glossary_entry_reference_commands
                .iter()
                .find(|cmd| cmd.name == ctx.parameter.name)
                .unwrap()
                .kind;

            let mut items = Vec::new();
            for doc in req.related() {
                if let DocumentContent::Latex(table) = &doc.content {
                    for entry in &table.glossary_entries {
                        match (cmd_kind, entry.kind) {
                            (Acronym, Acronym) | (General, General) | (General, Acronym) => {
                                let label = entry.label(&table).text().to_owned();
                                let text_edit = TextEdit::new(ctx.range, label.clone());
                                let item = factory::glossary_entry(req, label, text_edit);
                                items.push(item);
                            }
                            (Acronym, General) => {}
                        }
                    }
                }
            }
            items
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use texlab_feature::FeatureTester;
    use texlab_protocol::{CompletionTextEditExt, Range, RangeExt};

    #[tokio::test]
    async fn empty_latex_document() {
        let actual_items = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion(LatexGlossaryCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_items = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion(LatexGlossaryCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn acronym() {
        let actual_items = FeatureTester::new()
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
            .test_completion(LatexGlossaryCompletionProvider)
            .await;

        assert_eq!(actual_items.len(), 1);
        assert_eq!(actual_items[0].label, "lvm");
        assert_eq!(
            actual_items[0]
                .text_edit
                .as_ref()
                .and_then(|edit| edit.text_edit())
                .map(|edit| edit.range)
                .unwrap(),
            Range::new_simple(1, 9, 1, 12)
        );
    }
}
