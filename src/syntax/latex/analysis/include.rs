use crate::syntax::latex::ast::*;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LatexIncludeKind {
    Package,
    Class,
    TexFile,
    BibFile,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexInclude {
    pub command: Arc<LatexCommand>,
    pub kind: LatexIncludeKind,
}

impl LatexInclude {
    pub fn path(&self) -> &LatexToken {
        self.command.extract_word(0).unwrap()
    }

    pub fn new(command: Arc<LatexCommand>, kind: LatexIncludeKind) -> Self {
        LatexInclude { command, kind }
    }

    pub fn parse(commands: &[Arc<LatexCommand>]) -> (Vec<Self>, Vec<String>) {
        let mut includes = Vec::new();
        let mut components = Vec::new();
        for command in commands {
            let kind = match command.name.text() {
                "\\include" | "\\input" => Some(LatexIncludeKind::TexFile),
                "\\bibliography" | "\\addbibresource" => Some(LatexIncludeKind::BibFile),
                "\\usepackage" => Some(LatexIncludeKind::Package),
                "\\documentclass" => Some(LatexIncludeKind::Class),
                _ => None,
            };

            if let Some(kind) = kind {
                if command.has_word(0) {
                    let include = LatexInclude::new(Arc::clone(&command), kind);
                    match include.kind {
                        LatexIncludeKind::Package => {
                            components.push(format!("{}.sty", include.path().text()));
                        }
                        LatexIncludeKind::Class => {
                            components.push(format!("{}.cls", include.path().text()));
                        }
                        LatexIncludeKind::TexFile | LatexIncludeKind::BibFile => {}
                    }
                    includes.push(include);
                }
            }
        }
        (includes, components)
    }
}

#[cfg(test)]
mod tests {
    use crate::syntax::latex::LatexSyntaxTree;

    fn verify(text: &str, includes: Vec<&str>, components: Vec<&str>) {
        let tree = LatexSyntaxTree::from(text);
        let actual_includes: Vec<&str> = tree
            .includes
            .iter()
            .map(|include| include.path().text())
            .collect();
        assert_eq!(includes, actual_includes);
        assert_eq!(components, tree.components);
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
