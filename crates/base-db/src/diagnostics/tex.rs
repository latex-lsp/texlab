use rowan::{ast::AstNode, NodeOrToken, TextRange};
use syntax::latex;

use crate::{Config, Document, DocumentData};

use super::{Diagnostic, ErrorCode};

pub fn analyze(document: &mut Document, config: &Config) {
    if !document.uri.as_str().ends_with(".tex") {
        return;
    }

    let DocumentData::Tex(data) = &document.data else { return };

    let mut traversal = latex::SyntaxNode::new_root(data.green.clone()).preorder();
    while let Some(event) = traversal.next() {
        match event {
            rowan::WalkEvent::Enter(node) => {
                if let Some(environment) = latex::Environment::cast(node.clone()) {
                    if environment
                        .begin()
                        .and_then(|begin| begin.name())
                        .and_then(|name| name.key())
                        .map_or(false, |name| {
                            config
                                .syntax
                                .verbatim_environments
                                .contains(&name.to_string())
                        })
                    {
                        traversal.skip_subtree();
                        continue;
                    }
                }

                analyze_environment(document, node.clone())
                    .or_else(|| analyze_curly_group(document, node.clone(), config))
                    .or_else(|| analyze_curly_braces(document, node));
            }
            rowan::WalkEvent::Leave(_) => {
                continue;
            }
        };
    }
}

fn analyze_environment(document: &mut Document, node: latex::SyntaxNode) -> Option<()> {
    let environment = latex::Environment::cast(node)?;
    let begin = environment.begin()?.name()?.key()?;
    let end = environment.end()?.name()?.key()?;
    if begin != end {
        document.diagnostics.push(Diagnostic {
            range: latex::small_range(&begin),
            code: ErrorCode::MismatchedEnvironment,
        });
    }

    Some(())
}

fn analyze_curly_group(
    document: &mut Document,
    node: latex::SyntaxNode,
    config: &Config,
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
            config
                .syntax
                .verbatim_environments
                .contains(&name.to_string())
        });

    if !is_inside_verbatim_environment
        && !node
            .children_with_tokens()
            .filter_map(NodeOrToken::into_token)
            .any(|token| token.kind() == latex::R_CURLY)
    {
        document.diagnostics.push(Diagnostic {
            range: TextRange::empty(node.text_range().end()),
            code: ErrorCode::RCurlyInserted,
        });
    }

    Some(())
}

fn analyze_curly_braces(document: &mut Document, node: latex::SyntaxNode) -> Option<()> {
    if node.kind() == latex::ERROR && node.first_token()?.text() == "}" {
        document.diagnostics.push(Diagnostic {
            range: node.text_range(),
            code: ErrorCode::UnexpectedRCurly,
        });

        Some(())
    } else {
        None
    }
}
