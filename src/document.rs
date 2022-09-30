use std::{fmt, sync::Arc};

use derive_more::From;
use lsp_types::Url;

use crate::{
    line_index::LineIndex,
    parser::{parse_bibtex, parse_build_log, parse_latex},
    syntax::{
        latex::{self, LatexAnalyzerContext},
        BuildLog,
    },
    DocumentLanguage, Environment,
};

#[derive(Debug, Clone)]
pub struct LatexDocumentData {
    pub green: rowan::GreenNode,
    pub extras: Arc<latex::Extras>,
}

#[derive(Debug, Clone)]
pub struct BibtexDocumentData {
    pub green: rowan::GreenNode,
}

#[derive(Debug, Clone, From)]
pub enum DocumentData {
    Latex(Box<LatexDocumentData>),
    Bibtex(BibtexDocumentData),
    BuildLog(Arc<BuildLog>),
}

impl DocumentData {
    #[must_use]
    pub fn language(&self) -> DocumentLanguage {
        match self {
            Self::Latex(_) => DocumentLanguage::Latex,
            Self::Bibtex(_) => DocumentLanguage::Bibtex,
            Self::BuildLog(_) => DocumentLanguage::BuildLog,
        }
    }

    #[must_use]
    pub fn as_latex(&self) -> Option<&LatexDocumentData> {
        if let Self::Latex(data) = self {
            Some(data)
        } else {
            None
        }
    }

    #[must_use]
    pub fn as_bibtex(&self) -> Option<&BibtexDocumentData> {
        if let Self::Bibtex(data) = self {
            Some(data)
        } else {
            None
        }
    }

    #[must_use]
    pub fn as_build_log(&self) -> Option<&BuildLog> {
        if let Self::BuildLog(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct Document {
    uri: Arc<Url>,
    text: Arc<String>,
    line_index: Arc<LineIndex>,
    data: DocumentData,
}

impl fmt::Debug for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.uri)
    }
}

impl Document {
    pub fn uri(&self) -> &Arc<Url> {
        &self.uri
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn line_index(&self) -> &LineIndex {
        &self.line_index
    }

    pub fn data(&self) -> &DocumentData {
        &self.data
    }

    #[must_use]
    pub fn parse(
        environment: &Environment,
        uri: Arc<Url>,
        text: Arc<String>,
        language: DocumentLanguage,
    ) -> Self {
        let line_index = Arc::new(LineIndex::new(&text));
        let data = match language {
            DocumentLanguage::Latex => {
                let green = parse_latex(&text);
                let root = latex::SyntaxNode::new_root(green.clone());

                let base_uri = match &environment.options.root_directory {
                    Some(root_dir) => {
                        let root_dir = environment.current_directory.join(&root_dir);
                        Url::from_directory_path(root_dir)
                            .map_or_else(|()| Arc::clone(&uri), Arc::new)
                    }
                    None => Arc::clone(&uri),
                };

                let mut context = LatexAnalyzerContext {
                    environment,
                    extras: latex::Extras::default(),
                    document_uri: Arc::clone(&uri),
                    base_uri,
                };
                latex::analyze(&mut context, &root);
                let extras = Arc::new(context.extras);
                DocumentData::Latex(Box::new(LatexDocumentData { green, extras }))
            }
            DocumentLanguage::Bibtex => {
                let green = parse_bibtex(&text);
                DocumentData::Bibtex(BibtexDocumentData { green })
            }
            DocumentLanguage::BuildLog => {
                let data = Arc::new(parse_build_log(&text));
                DocumentData::BuildLog(data)
            }
        };

        Self {
            uri,
            text,
            line_index,
            data,
        }
    }
}
