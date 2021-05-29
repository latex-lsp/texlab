use std::sync::Arc;

use cstree::TextRange;
use lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString};
use multimap::MultiMap;

use crate::{
    syntax::{latex, CstNode},
    Document, LineIndexExt, Uri, Workspace,
};

pub fn analyze_latex_static(
    workspace: &dyn Workspace,
    diagnostics_by_uri: &mut MultiMap<Arc<Uri>, Diagnostic>,
    uri: &Uri,
) -> Option<()> {
    let document = workspace.get(uri)?;
    if !document.uri.as_str().ends_with(".tex") {
        return None;
    }

    let data = document.data.as_latex()?;

    for node in data.root.descendants() {
        analyze_environment(&document, diagnostics_by_uri, node)
            .or_else(|| analyze_curly_group(&document, diagnostics_by_uri, node))
            .or_else(|| {
                if node.kind() == latex::ERROR && node.first_token()?.text() == "}" {
                    diagnostics_by_uri.insert(
                        Arc::clone(&document.uri),
                        Diagnostic {
                            range: document.line_index.line_col_lsp_range(node.text_range()),
                            severity: Some(DiagnosticSeverity::Error),
                            code: Some(NumberOrString::Number(1)),
                            code_description: None,
                            source: Some("texlab".to_string()),
                            message: "Unexpected \"}\"".to_string(),
                            related_information: None,
                            tags: None,
                            data: None,
                        },
                    );
                    Some(())
                } else {
                    None
                }
            });
    }

    Some(())
}

fn analyze_environment(
    document: &Document,
    diagnostics_by_uri: &mut MultiMap<Arc<Uri>, Diagnostic>,
    node: &latex::SyntaxNode,
) -> Option<()> {
    let environment = latex::Environment::cast(node)?;
    let name1 = environment.begin()?.name()?.key()?;
    let name2 = environment.end()?.name()?.key()?;
    if name1 != name2 {
        diagnostics_by_uri.insert(
            Arc::clone(&document.uri),
            Diagnostic {
                range: document.line_index.line_col_lsp_range(name1.small_range()),
                severity: Some(DiagnosticSeverity::Error),
                code: Some(NumberOrString::Number(3)),
                code_description: None,
                source: Some("texlab".to_string()),
                message: "Mismatched environment".to_string(),
                related_information: None,
                tags: None,
                data: None,
            },
        );
    }
    Some(())
}

fn analyze_curly_group(
    document: &Document,
    diagnostics_by_uri: &mut MultiMap<Arc<Uri>, Diagnostic>,
    node: &latex::SyntaxNode,
) -> Option<()> {
    if !matches!(
        node.kind(),
        latex::CURLY_GROUP
            | latex::CURLY_GROUP_COMMAND
            | latex::CURLY_GROUP_KEY_VALUE
            | latex::CURLY_GROUP_WORD
            | latex::CURLY_GROUP_WORD_LIST
    ) {
        return None;
    }

    let is_inside_verbatim_environment = node
        .ancestors()
        .filter_map(latex::Environment::cast)
        .filter_map(|env| env.begin())
        .filter_map(|begin| begin.name())
        .filter_map(|name| name.key())
        .any(|name| {
            ["asy", "lstlisting", "minted", "verbatim"].contains(&name.to_string().as_str())
        });

    if !is_inside_verbatim_environment
        && !node
            .children_with_tokens()
            .filter_map(|element| element.into_token())
            .any(|token| token.kind() == latex::R_CURLY)
    {
        diagnostics_by_uri.insert(
            Arc::clone(&document.uri),
            Diagnostic {
                range: document
                    .line_index
                    .line_col_lsp_range(TextRange::empty(node.text_range().end())),
                severity: Some(DiagnosticSeverity::Error),
                code: Some(NumberOrString::Number(2)),
                code_description: None,
                source: Some("texlab".to_string()),
                message: "Missing \"}\" inserted".to_string(),
                related_information: None,
                tags: None,
                data: None,
            },
        );
    }

    Some(())
}
