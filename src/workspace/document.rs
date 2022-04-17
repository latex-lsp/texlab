use std::{fmt, sync::Arc};

use derive_more::From;

use crate::{
    line_index::LineIndex,
    syntax::{
        bibtex, build_log,
        latex::{self, LatexAnalyzerContext},
    },
    DocumentLanguage, ServerContext, Uri,
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
    BuildLog(Arc<build_log::Parse>),
}

impl DocumentData {
    pub fn language(&self) -> DocumentLanguage {
        match self {
            Self::Latex(_) => DocumentLanguage::Latex,
            Self::Bibtex(_) => DocumentLanguage::Bibtex,
            Self::BuildLog(_) => DocumentLanguage::BuildLog,
        }
    }

    pub fn as_latex(&self) -> Option<&LatexDocumentData> {
        if let Self::Latex(data) = self {
            Some(data)
        } else {
            None
        }
    }

    pub fn as_bibtex(&self) -> Option<&BibtexDocumentData> {
        if let Self::Bibtex(data) = self {
            Some(data)
        } else {
            None
        }
    }

    pub fn as_build_log(&self) -> Option<&build_log::Parse> {
        if let Self::BuildLog(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct Document {
    pub uri: Arc<Uri>,
    pub text: Arc<String>,
    pub line_index: Arc<LineIndex>,
    pub data: DocumentData,
}

impl fmt::Debug for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.uri)
    }
}

impl Document {
    pub fn parse(
        context: Arc<ServerContext>,
        uri: Arc<Uri>,
        text: Arc<String>,
        language: DocumentLanguage,
    ) -> Self {
        let line_index = Arc::new(LineIndex::new(&text));
        let data = match language {
            DocumentLanguage::Latex => {
                let green = latex::parse(&text).green;
                let root = latex::SyntaxNode::new_root(green.clone());

                let base_uri = match &context.options.read().unwrap().root_directory {
                    Some(root_dir) => {
                        let root_dir = context.current_directory.join(&root_dir);
                        Uri::from_directory_path(root_dir)
                            .map(Arc::new)
                            .unwrap_or_else(|()| Arc::clone(&uri))
                    }
                    None => Arc::clone(&uri),
                };

                let mut context = LatexAnalyzerContext {
                    inner: context,
                    extras: latex::Extras::default(),
                    document_uri: Arc::clone(&uri),
                    base_uri,
                };
                latex::analyze(&mut context, &root);
                let extras = Arc::new(context.extras);
                DocumentData::Latex(Box::new(LatexDocumentData { green, extras }))
            }
            DocumentLanguage::Bibtex => {
                let green = bibtex::parse(&text).green;
                DocumentData::Bibtex(BibtexDocumentData { green })
            }
            DocumentLanguage::BuildLog => {
                let data = Arc::new(build_log::parse(&text));
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

    pub fn language(&self) -> DocumentLanguage {
        self.data.language()
    }
}
