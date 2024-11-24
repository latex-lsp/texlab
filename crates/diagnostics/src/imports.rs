use base_db::{semantics::tex::Link, util::queries, Document, Workspace};
use itertools::Itertools;
use rustc_hash::FxHashMap;
use syntax::latex::{self, SyntaxKind};
use url::Url;

use crate::{types::Diagnostic, ImportError};

pub(crate) fn update(
    document: &Document,
    _config: &base_db::Config,
    _imports: &mut std::collections::HashMap<Url, Vec<Diagnostic>, rustc_hash::FxBuildHasher>,
) -> Option<()> {
    let root = document.data.as_tex()?.root_node();
    let mut analyzer = ImportAnalyzer {
        imports: Vec::new(),
        diagnostics: Vec::new(),
    };
    analyzer.visit(&root);

    Some(())
}

struct ImportAnalyzer {
    imports: Vec<latex::SyntaxNode>,
    diagnostics: Vec<Diagnostic>,
}

impl ImportAnalyzer {
    fn visit(&mut self, node: &latex::SyntaxNode) {
        match node.kind() {
            latex::SyntaxKind::ROOT => {
                let preamble = node
                    .children()
                    .filter(|child| child.kind() == SyntaxKind::PREAMBLE)
                    .exactly_one()
                    .unwrap();

                self.visit(&preamble);
            }
            latex::SyntaxKind::PREAMBLE => {
                for child in node.children() {
                    if child.kind() == SyntaxKind::PACKAGE_INCLUDE {
                        self.visit(&child);
                    }
                }
            }
            latex::SyntaxKind::PACKAGE_INCLUDE => {
                for child in node.children() {
                    if child.kind() == SyntaxKind::CURLY_GROUP_WORD_LIST {
                        self.visit(&child);
                    }
                }
            }
            latex::SyntaxKind::CURLY_GROUP_WORD_LIST => {
                for child in node.children() {
                    if child.kind() == SyntaxKind::KEY {
                        self.visit(&child);
                    }
                }
            }
            latex::SyntaxKind::KEY => {
                self.imports.push(node.clone());
            }
            _ => (),
        }
    }

    fn find_duplicates(&mut self) -> Option<()> {
        let len = self.imports.len();
        for i in 0..len {
            for j in i..len {
                if self.imports[i].text().to_string() == self.imports[j].text().to_string() {
                    self.diagnostics.push(Diagnostic::Import(
                        self.imports[j].text_range(),
                        ImportError::DuplicateImport(vec![]),
                    ));
                }
            }
        }

        Some(())
    }
}

pub fn detect_duplicate_imports(
    workspace: &Workspace,
    results: &mut FxHashMap<Url, Vec<Diagnostic>>,
) {
    log::debug!("IMPORT CONFLICT CHECK");
    for conflict in queries::Conflict::find_all::<Link>(workspace) {
        log::debug!("IMPORT {:?}", conflict);
        let others = conflict
            .rest
            .iter()
            .map(|location| (location.document.uri.clone(), location.range))
            .collect();

        let diagnostic =
            Diagnostic::Import(conflict.main.range, ImportError::DuplicateImport(others));
        results
            .entry(conflict.main.document.uri.clone())
            .or_default()
            .push(diagnostic);
    }
}
