use base_db::{Config, Document, DocumentData, Workspace};
use rowan::{ast::AstNode, NodeOrToken, TextRange};
use syntax::latex;

use crate::{
    types::{DiagnosticData, TexError},
    util::SimpleDiagnosticSource,
    Diagnostic, DiagnosticBuilder, DiagnosticSource,
};

#[derive(Default)]
pub struct TexSyntaxErrors(SimpleDiagnosticSource);

impl DiagnosticSource for TexSyntaxErrors {
    fn update(&mut self, workspace: &Workspace, document: &Document) {
        let mut analyzer = Analyzer {
            document,
            config: workspace.config(),
            diagnostics: Vec::new(),
        };

        analyzer.analyze_root();
        self.0
            .errors
            .insert(document.uri.clone(), analyzer.diagnostics);
    }

    fn publish<'db>(
        &'db mut self,
        workspace: &'db Workspace,
        builder: &mut DiagnosticBuilder<'db>,
    ) {
        self.0.publish(workspace, builder);
    }
}

struct Analyzer<'a> {
    document: &'a Document,
    config: &'a Config,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Analyzer<'a> {
    fn analyze_root(&mut self) {
        if !self.document.uri.as_str().ends_with(".tex") {
            return;
        }

        let DocumentData::Tex(data) = &self.document.data else { return };

        let verbatim_envs = &self.config.syntax.verbatim_environments;

        let mut traversal = latex::SyntaxNode::new_root(data.green.clone()).preorder();
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
            self.diagnostics.push(Diagnostic {
                range: latex::small_range(&begin),
                data: DiagnosticData::Tex(TexError::MismatchedEnvironment),
            });
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
            self.diagnostics.push(Diagnostic {
                range: TextRange::empty(node.text_range().end()),
                data: DiagnosticData::Tex(TexError::ExpectingRCurly),
            });
        }

        Some(())
    }

    fn analyze_curly_braces(&mut self, node: latex::SyntaxNode) -> Option<()> {
        if node.kind() == latex::ERROR && node.first_token()?.text() == "}" {
            self.diagnostics.push(Diagnostic {
                range: node.text_range(),
                data: DiagnosticData::Tex(TexError::UnexpectedRCurly),
            });

            Some(())
        } else {
            None
        }
    }
}
