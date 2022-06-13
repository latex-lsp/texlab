use std::sync::Arc;

use dashmap::DashMap;
use lsp_types::{DiagnosticSeverity, Url};
use rowan::{ast::AstNode, NodeOrToken, TextRange};

use crate::{syntax::latex, Document, LineIndexExt, Workspace};

use super::{Diagnostic, DiagnosticCode, LatexCode};

pub fn collect_latex_diagnostics(
    all_diagnostics: &DashMap<Arc<Url>, Vec<Diagnostic>>,
    workspace: &Workspace,
    uri: &Url,
) -> Option<()> {
    let document = workspace.documents_by_uri.get(uri)?;
    if !document.uri.as_str().ends_with(".tex") {
        return None;
    }

    let data = document.data.as_latex()?;

    all_diagnostics.alter(uri, |_, mut diagnostics| {
        diagnostics.retain(|diag| !matches!(diag.code, DiagnosticCode::Latex(_)));
        diagnostics
    });

    for node in latex::SyntaxNode::new_root(data.green.clone()).descendants() {
        analyze_environment(all_diagnostics, document, node.clone())
            .or_else(|| analyze_curly_group(all_diagnostics, document, &node))
            .or_else(|| {
                if node.kind() == latex::ERROR && node.first_token()?.text() == "}" {
                    let code = LatexCode::UnexpectedRCurly;
                    all_diagnostics
                        .entry(Arc::clone(&document.uri))
                        .or_default()
                        .push(Diagnostic {
                            severity: DiagnosticSeverity::ERROR,
                            range: document.line_index.line_col_lsp_range(node.text_range()),
                            code: DiagnosticCode::Latex(code),
                            message: String::from(code),
                        });

                    Some(())
                } else {
                    None
                }
            });
    }

    Some(())
}

fn analyze_environment(
    all_diagnostics: &DashMap<Arc<Url>, Vec<Diagnostic>>,
    document: &Document,
    node: latex::SyntaxNode,
) -> Option<()> {
    let environment = latex::Environment::cast(node)?;
    let name1 = environment.begin()?.name()?.key()?;
    let name2 = environment.end()?.name()?.key()?;
    if name1 != name2 {
        let code = LatexCode::MismatchedEnvironment;
        all_diagnostics
            .entry(Arc::clone(&document.uri))
            .or_default()
            .push(Diagnostic {
                severity: DiagnosticSeverity::ERROR,
                range: document
                    .line_index
                    .line_col_lsp_range(latex::small_range(&name1)),
                code: DiagnosticCode::Latex(code),
                message: String::from(code),
            });
    }
    Some(())
}

fn analyze_curly_group(
    all_diagnostics: &DashMap<Arc<Url>, Vec<Diagnostic>>,
    document: &Document,
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
            .filter_map(NodeOrToken::into_token)
            .any(|token| token.kind() == latex::R_CURLY)
    {
        let code = LatexCode::RCurlyInserted;
        all_diagnostics
            .entry(Arc::clone(&document.uri))
            .or_default()
            .push(Diagnostic {
                severity: DiagnosticSeverity::ERROR,
                range: document
                    .line_index
                    .line_col_lsp_range(TextRange::empty(node.text_range().end())),
                code: DiagnosticCode::Latex(code),
                message: String::from(code),
            });
    }

    Some(())
}
