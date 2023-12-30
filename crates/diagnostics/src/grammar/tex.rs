use base_db::{Config, Document, TexDocumentData};
use multimap::MultiMap;
use rowan::{ast::AstNode, NodeOrToken, TextRange};
use syntax::latex;
use url::Url;

use crate::types::{Diagnostic, TexError};

pub fn update(
    document: &Document,
    config: &Config,
    results: &mut MultiMap<Url, Diagnostic>,
) -> Option<()> {
    let data = document.data.as_tex()?;
    if !document.uri.as_str().ends_with(".tex") {
        return None;
    }

    let mut analyzer = Analyzer {
        data,
        config,
        diagnostics: Vec::new(),
    };

    analyzer.analyze_root();

    *results
        .entry(document.uri.clone())
        .or_insert_vec(Vec::new()) = analyzer.diagnostics;

    Some(())
}

struct Analyzer<'a> {
    data: &'a TexDocumentData,
    config: &'a Config,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Analyzer<'a> {
    fn analyze_root(&mut self) {
        let verbatim_envs = &self.config.syntax.verbatim_environments;

        let mut traversal = self.data.root_node().preorder();
        while let Some(event) = traversal.next() {
            match event {
                rowan::WalkEvent::Enter(node) => {
                    if let Some(environment) = latex::Environment::cast(node.clone()) {
                        if environment
                            .begin()
                            .and_then(|begin| begin.name())
                            .and_then(|name| name.key())
                            .map_or(false, |name| verbatim_envs.contains(&name.to_string()))
                        {
                            traversal.skip_subtree();
                            continue;
                        }
                    }

                    self.analyze_environment(node.clone())
                        .or_else(|| self.analyze_curly_group(node.clone()))
                        .or_else(|| self.analyze_curly_braces(node));
                }
                rowan::WalkEvent::Leave(_) => {
                    continue;
                }
            };
        }
    }

    fn analyze_environment(&mut self, node: latex::SyntaxNode) -> Option<()> {
        let environment = latex::Environment::cast(node)?;
        let begin = environment.begin()?.name()?.key()?;
        let end = environment.end()?.name()?.key()?;
        if begin != end {
            self.diagnostics.push(Diagnostic::Tex(
                latex::small_range(&begin),
                TexError::MismatchedEnvironment,
            ));
        }

        Some(())
    }

    fn analyze_curly_group(&mut self, node: latex::SyntaxNode) -> Option<()> {
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

        if !node
            .children_with_tokens()
            .filter_map(NodeOrToken::into_token)
            .any(|token| token.kind() == latex::R_CURLY)
        {
            self.diagnostics.push(Diagnostic::Tex(
                TextRange::empty(node.text_range().end()),
                TexError::ExpectingRCurly,
            ));
        }

        Some(())
    }

    fn analyze_curly_braces(&mut self, node: latex::SyntaxNode) -> Option<()> {
        if node.kind() == latex::ERROR && node.first_token()?.text() == "}" {
            self.diagnostics.push(Diagnostic::Tex(
                node.text_range(),
                TexError::UnexpectedRCurly,
            ));

            Some(())
        } else {
            None
        }
    }
}
