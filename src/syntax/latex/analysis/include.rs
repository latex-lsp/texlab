use crate::syntax::latex::ast::*;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LatexIncludeKind {
    Package,
    Class,
    TexFile,
    BibFile,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexInclude<'a> {
    pub command: &'a LatexCommand,
    pub path: &'a LatexToken,
    pub kind: LatexIncludeKind,
}

impl<'a> LatexInclude<'a> {
    pub fn new(command: &'a LatexCommand, path: &'a LatexToken, kind: LatexIncludeKind) -> Self {
        LatexInclude {
            command,
            path,
            kind,
        }
    }
}

pub struct LatexIncludeAnalyzer<'a> {
    pub included_files: Vec<LatexInclude<'a>>,
    pub included_components: Vec<String>,
}

impl<'a> LatexIncludeAnalyzer<'a> {
    pub fn new() -> Self {
        LatexIncludeAnalyzer {
            included_files: Vec::new(),
            included_components: Vec::new(),
        }
    }

    fn register_component(&mut self, include: &LatexInclude) {
        let path = include.path.text();
        match include.kind {
            LatexIncludeKind::Package => self.included_components.push(format!("{}.sty", path)),
            LatexIncludeKind::Class => self.included_components.push(format!("{}.cls", path)),
            LatexIncludeKind::TexFile | LatexIncludeKind::BibFile => (),
        }
    }
}

impl<'a> LatexVisitor<'a> for LatexIncludeAnalyzer<'a> {
    fn visit_root(&mut self, root: &'a LatexRoot) {
        LatexWalker::walk_root(self, root);
    }

    fn visit_group(&mut self, group: &'a LatexGroup) {
        LatexWalker::walk_group(self, group);
    }

    fn visit_command(&mut self, command: &'a LatexCommand) {
        let kind = match command.name.text() {
            "\\include" | "\\input" => Some(LatexIncludeKind::TexFile),
            "\\bibliography" | "\\addbibresource" => Some(LatexIncludeKind::BibFile),
            "\\usepackage" => Some(LatexIncludeKind::Package),
            "\\documentclass" => Some(LatexIncludeKind::Class),
            _ => None,
        };

        if let Some(kind) = kind {
            if let Some(path) = command.extract_word(0) {
                let include = LatexInclude::new(command, path, kind);
                self.register_component(&include);
                self.included_files.push(include);
            }
        }
        LatexWalker::walk_command(self, command);
    }

    fn visit_text(&mut self, text: &'a LatexText) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::latex::LatexSyntaxTree;

    fn verify(text: &str, includes: Vec<&str>, components: Vec<&str>) {
        let tree = LatexSyntaxTree::from(text);
        let mut analyzer = LatexIncludeAnalyzer::new();
        analyzer.visit_root(&tree.root);
        let actual_includes: Vec<&str> = analyzer
            .included_files
            .iter()
            .map(|include| include.path.text())
            .collect();
        assert_eq!(includes, actual_includes);
        assert_eq!(components, analyzer.included_components);
    }

    #[test]
    fn test_valid() {
        verify("\\include{foo}", vec!["foo"], vec![]);
        verify("\\bibliography{foo}", vec!["foo"], vec![]);
        verify(
            "\\usepackage{amsmath}",
            vec!["amsmath"],
            vec!["amsmath.sty"],
        );
        verify(
            "\\documentclass{article}",
            vec!["article"],
            vec!["article.cls"],
        );
    }

    #[test]
    fn test_invalid() {
        verify("\\include", vec![], vec![]);
        verify("\\include{}", vec![], vec![]);
        verify("\\include{foo bar}", vec![], vec![]);
    }

    #[test]
    fn test_unrelated() {
        verify("\\foo", vec![], vec![]);
        verify("\\foo{bar}", vec![], vec![]);
    }
}
