use texlab_protocol::{Diagnostic, DiagnosticSeverity, Position, Range};
use texlab_syntax::*;
use texlab_workspace::Document;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BibtexErrorCode {
    MissingBeginBrace,
    MissingEntryKey,
    MissingComma,
    MissingEndBrace,
    MissingAssign,
    MissingContent,
    MissingQuote,
}

impl BibtexErrorCode {
    pub fn message(self) -> &'static str {
        match self {
            BibtexErrorCode::MissingBeginBrace => "Expecting a curly bracket: \"{\"",
            BibtexErrorCode::MissingEntryKey => "Expecting an entry key",
            BibtexErrorCode::MissingComma => "Expecting a comma: \",\"",
            BibtexErrorCode::MissingEndBrace => "Expecting a curly bracket: \"}\"",
            BibtexErrorCode::MissingAssign => "Expecting an equals sign: \"=\"",
            BibtexErrorCode::MissingContent => "Expecting content",
            BibtexErrorCode::MissingQuote => "Expecting a quote: '\"'",
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BibtexError {
    code: BibtexErrorCode,
    position: Position,
}

impl BibtexError {
    pub fn new(code: BibtexErrorCode, position: Position) -> Self {
        Self { code, position }
    }

    pub fn analyze(tree: &BibtexSyntaxTree) -> Vec<Self> {
        let mut errors = Vec::new();
        for entry in tree.entries() {
            if entry.is_comment() {
                continue;
            }

            if entry.left.is_none() {
                errors.push(BibtexError::new(
                    BibtexErrorCode::MissingBeginBrace,
                    entry.ty.end(),
                ));
                continue;
            }

            if entry.key.is_none() {
                errors.push(BibtexError::new(
                    BibtexErrorCode::MissingEntryKey,
                    entry.left.as_ref().unwrap().end(),
                ));
                continue;
            }

            if entry.comma.is_none() {
                errors.push(BibtexError::new(
                    BibtexErrorCode::MissingComma,
                    entry.key.as_ref().unwrap().end(),
                ));
                continue;
            }

            for i in 0..entry.fields.len() {
                let field = &entry.fields[i];
                if field.assign.is_none() {
                    errors.push(BibtexError::new(
                        BibtexErrorCode::MissingAssign,
                        field.name.end(),
                    ));
                    continue;
                }

                if field.content.is_none() {
                    errors.push(BibtexError::new(
                        BibtexErrorCode::MissingContent,
                        field.assign.as_ref().unwrap().end(),
                    ));
                    continue;
                }

                Self::analyze_content(&mut errors, &field.content.as_ref().unwrap());

                if i != entry.fields.len() - 1 && field.comma.is_none() {
                    errors.push(BibtexError::new(
                        BibtexErrorCode::MissingComma,
                        field.content.as_ref().unwrap().end(),
                    ));
                    continue;
                }
            }

            if entry.right.is_none() {
                errors.push(BibtexError::new(
                    BibtexErrorCode::MissingEndBrace,
                    entry.end(),
                ));
                continue;
            }
        }
        errors
    }

    fn analyze_content(mut errors: &mut Vec<BibtexError>, content: &BibtexContent) {
        match content {
            BibtexContent::QuotedContent(content) => {
                for child in &content.children {
                    Self::analyze_content(&mut errors, &child);
                }

                if content.right.is_none() {
                    errors.push(BibtexError::new(
                        BibtexErrorCode::MissingQuote,
                        content.end(),
                    ));
                }
            }
            BibtexContent::BracedContent(content) => {
                for child in &content.children {
                    Self::analyze_content(&mut errors, &child);
                }

                if content.right.is_none() {
                    errors.push(BibtexError::new(
                        BibtexErrorCode::MissingEndBrace,
                        content.end(),
                    ));
                }
            }
            BibtexContent::Concat(concat) => {
                Self::analyze_content(&mut errors, &concat.left);
                match &concat.right {
                    Some(right) => {
                        Self::analyze_content(&mut errors, right);
                    }
                    None => {
                        errors.push(BibtexError::new(
                            BibtexErrorCode::MissingContent,
                            concat.end(),
                        ));
                    }
                }
            }
            BibtexContent::Word(_) | BibtexContent::Command(_) => {}
        }
    }
}

impl Into<Diagnostic> for BibtexError {
    fn into(self) -> Diagnostic {
        Diagnostic {
            source: Some("bibtex".into()),
            range: Range::new(self.position, self.position),
            message: self.code.message().into(),
            severity: Some(DiagnosticSeverity::Error),
            code: None,
            related_information: None,
            tags: None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct BibtexDiagnosticsProvider;

impl BibtexDiagnosticsProvider {
    pub fn get(self, document: &Document) -> Vec<Diagnostic> {
        if let SyntaxTree::Bibtex(tree) = &document.tree {
            BibtexError::analyze(&tree)
                .into_iter()
                .map(Into::into)
                .collect()
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn begin_brace() {
        let errors = BibtexError::analyze(&"@article".into());
        assert_eq!(
            errors,
            vec![BibtexError::new(
                BibtexErrorCode::MissingBeginBrace,
                Position::new(0, 8),
            )]
        );
    }

    #[test]
    fn entry_key() {
        let errors = BibtexError::analyze(&"@article{".into());
        assert_eq!(
            errors,
            vec![BibtexError::new(
                BibtexErrorCode::MissingEntryKey,
                Position::new(0, 9),
            )]
        );
    }

    #[test]
    fn entry_comma() {
        let errors = BibtexError::analyze(&"@article{foo".into());
        assert_eq!(
            errors,
            vec![BibtexError::new(
                BibtexErrorCode::MissingComma,
                Position::new(0, 12),
            )]
        );
    }

    #[test]
    fn entry_end_brace() {
        let errors = BibtexError::analyze(&"@article{foo,".into());
        assert_eq!(
            errors,
            vec![BibtexError::new(
                BibtexErrorCode::MissingEndBrace,
                Position::new(0, 13),
            )]
        );
    }

    #[test]
    fn field_equals() {
        let errors = BibtexError::analyze(&"@article{foo, bar}".into());
        assert_eq!(
            errors,
            vec![BibtexError::new(
                BibtexErrorCode::MissingAssign,
                Position::new(0, 17),
            )]
        );
    }

    #[test]
    fn field_content() {
        let errors = BibtexError::analyze(&"@article{foo,\nbar = }".into());
        assert_eq!(
            errors,
            vec![BibtexError::new(
                BibtexErrorCode::MissingContent,
                Position::new(1, 5),
            )]
        );
    }

    #[test]
    fn field_comma() {
        let text = "@article{foo,\nfoo = bar\nbaz = qux}";
        let errors = BibtexError::analyze(&text.into());
        assert_eq!(
            errors,
            vec![BibtexError::new(
                BibtexErrorCode::MissingComma,
                Position::new(1, 9),
            )]
        );
    }

    #[test]
    fn content_quote() {
        let text = "@article{foo, bar =\n\"}";
        let errors = BibtexError::analyze(&text.into());
        assert_eq!(
            errors,
            vec![BibtexError::new(
                BibtexErrorCode::MissingQuote,
                Position::new(1, 1),
            )]
        );
    }

    #[test]
    fn content_brace() {
        let text = "@article{foo, bar =\n{";
        let errors = BibtexError::analyze(&text.into());
        assert_eq!(
            errors,
            vec![
                BibtexError::new(BibtexErrorCode::MissingEndBrace, Position::new(1, 1)),
                BibtexError::new(BibtexErrorCode::MissingEndBrace, Position::new(1, 1)),
            ]
        );
    }

    #[test]
    fn content_concat() {
        let text = "@article{foo, bar = baz \n# }";
        let errors = BibtexError::analyze(&text.into());
        assert_eq!(
            errors,
            vec![BibtexError::new(
                BibtexErrorCode::MissingContent,
                Position::new(1, 1)
            )]
        );
    }

    #[test]
    fn entry_valid() {
        let text = "@article{foo, bar = \"baz {qux}\" # quux}";
        let errors = BibtexError::analyze(&text.into());
        assert_eq!(errors, Vec::new());
    }
}
