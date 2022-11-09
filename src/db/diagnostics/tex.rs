use lsp_types::DiagnosticSeverity;
use rowan::{ast::AstNode, NodeOrToken, TextRange};

use crate::{db::document::Document, syntax::latex, util::line_index_ext::LineIndexExt, Db};

use super::{Diagnostic, DiagnosticCode, TexCode};

#[salsa::tracked(return_ref)]
pub fn collect(db: &dyn Db, document: Document) -> Vec<Diagnostic> {
    let mut results = Vec::new();

    if !document.location(db).uri(db).as_str().ends_with(".tex") {
        return results;
    }

    let data = match document.parse(db).as_tex() {
        Some(data) => data,
        None => return results,
    };

    for node in data.root(db).descendants() {
        analyze_environment(db, document, node.clone(), &mut results)
            .or_else(|| analyze_curly_group(db, document, node.clone(), &mut results))
            .or_else(|| analyze_curly_braces(document, db, node, &mut results));
    }

    results
}

fn analyze_environment(
    db: &dyn Db,
    document: Document,
    node: latex::SyntaxNode,
    results: &mut Vec<Diagnostic>,
) -> Option<()> {
    let environment = latex::Environment::cast(node)?;
    let name1 = environment.begin()?.name()?.key()?;
    let name2 = environment.end()?.name()?.key()?;
    if name1 != name2 {
        let code = TexCode::MismatchedEnvironment;
        results.push(Diagnostic {
            severity: DiagnosticSeverity::ERROR,
            range: document
                .contents(db)
                .line_index(db)
                .line_col_lsp_range(latex::small_range(&name1)),
            code: DiagnosticCode::Tex(code),
            message: String::from(code),
        });
    }

    Some(())
}

fn analyze_curly_group(
    db: &dyn Db,
    document: Document,
    node: latex::SyntaxNode,
    results: &mut Vec<Diagnostic>,
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
        let code = TexCode::RCurlyInserted;
        results.push(Diagnostic {
            severity: DiagnosticSeverity::ERROR,
            range: document
                .contents(db)
                .line_index(db)
                .line_col_lsp_range(TextRange::empty(node.text_range().end())),
            code: DiagnosticCode::Tex(code),
            message: String::from(code),
        });
    }

    Some(())
}

fn analyze_curly_braces(
    document: Document,
    db: &dyn Db,
    node: rowan::SyntaxNode<latex::LatexLanguage>,
    results: &mut Vec<Diagnostic>,
) -> Option<()> {
    if node.kind() == latex::ERROR && node.first_token()?.text() == "}" {
        let code = TexCode::UnexpectedRCurly;
        results.push(Diagnostic {
            severity: DiagnosticSeverity::ERROR,
            range: document
                .contents(db)
                .line_index(db)
                .line_col_lsp_range(node.text_range()),
            code: DiagnosticCode::Tex(code),
            message: String::from(code),
        });

        Some(())
    } else {
        None
    }
}
