use std::collections::HashMap;

use lsp_types::{Range, RenameParams, TextEdit, WorkspaceEdit};
use rowan::ast::AstNode;

use crate::{
    features::cursor::{CursorContext, HasPosition},
    syntax::latex,
    LineIndexExt,
};

pub fn prepare_label_rename<P: HasPosition>(context: &CursorContext<P>) -> Option<Range> {
    let (_, range) = context.find_label_name_key()?;

    Some(
        context
            .request
            .main_document()
            .line_index
            .line_col_lsp_range(range),
    )
}

pub fn rename_label(context: &CursorContext<RenameParams>) -> Option<WorkspaceEdit> {
    prepare_label_rename(context)?;
    let (name_text, _) = context.find_label_name_key()?;

    let mut changes = HashMap::new();
    for document in context.request.workspace.documents_by_uri.values() {
        if let Some(data) = document.data.as_latex() {
            let mut edits = Vec::new();
            for node in latex::SyntaxNode::new_root(data.green.clone()).descendants() {
                if let Some(range) = latex::LabelDefinition::cast(node.clone())
                    .and_then(|label| label.name())
                    .and_then(|name| name.key())
                    .filter(|name| name.to_string() == name_text)
                    .map(|name| {
                        document
                            .line_index
                            .line_col_lsp_range(latex::small_range(&name))
                    })
                {
                    edits.push(TextEdit::new(
                        range,
                        context.request.params.new_name.clone(),
                    ));
                }

                latex::LabelReference::cast(node.clone())
                    .and_then(|label| label.name_list())
                    .into_iter()
                    .flat_map(|label| label.keys())
                    .filter(|name| name.to_string() == name_text)
                    .map(|name| {
                        document
                            .line_index
                            .line_col_lsp_range(latex::small_range(&name))
                    })
                    .for_each(|range| {
                        edits.push(TextEdit::new(
                            range,
                            context.request.params.new_name.clone(),
                        ));
                    });

                if let Some(label) = latex::LabelReferenceRange::cast(node.clone()) {
                    if let Some(name1) = label
                        .from()
                        .and_then(|name| name.key())
                        .filter(|name| name.to_string() == name_text)
                    {
                        edits.push(TextEdit::new(
                            document
                                .line_index
                                .line_col_lsp_range(latex::small_range(&name1)),
                            context.request.params.new_name.clone(),
                        ));
                    }

                    if let Some(name2) = label
                        .from()
                        .and_then(|name| name.key())
                        .filter(|name| name.to_string() == name_text)
                    {
                        edits.push(TextEdit::new(
                            document
                                .line_index
                                .line_col_lsp_range(latex::small_range(&name2)),
                            context.request.params.new_name.clone(),
                        ));
                    }
                }
            }

            changes.insert(document.uri.as_ref().clone().into(), edits);
        }
    }

    Some(WorkspaceEdit::new(changes))
}

#[cfg(test)]
mod tests {
    use crate::{features::testing::FeatureTester, RangeExt};

    use super::*;

    #[test]
    fn test_label() {
        let tester = FeatureTester::builder()
            .files(vec![
                ("foo.tex", r#"\label{foo}\include{bar}"#),
                ("bar.tex", r#"\ref{foo}"#),
                ("baz.tex", r#"\ref{foo}"#),
            ])
            .main("foo.tex")
            .line(0)
            .character(7)
            .new_name("bar")
            .build();

        let uri1 = tester.uri("foo.tex");
        let uri2 = tester.uri("bar.tex");
        let request = tester.rename();

        let context = CursorContext::new(request);
        let actual_edit = rename_label(&context).unwrap();

        let mut expected_changes = HashMap::new();
        expected_changes.insert(
            uri1.as_ref().clone().into(),
            vec![TextEdit::new(Range::new_simple(0, 7, 0, 10), "bar".into())],
        );
        expected_changes.insert(
            uri2.as_ref().clone().into(),
            vec![TextEdit::new(Range::new_simple(0, 5, 0, 8), "bar".into())],
        );
        let expected_edit = WorkspaceEdit::new(expected_changes);

        assert_eq!(actual_edit, expected_edit);
    }
}
