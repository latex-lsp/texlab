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
    pub extras: latex::Extras,
}

#[derive(Debug, Clone)]
pub struct BibtexDocumentData {
    pub green: rowan::GreenNode,
}

#[derive(Debug, Clone, From)]
pub enum DocumentData {
    Latex(Box<LatexDocumentData>),
    Bibtex(BibtexDocumentData),
    BuildLog(build_log::Parse),
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
    pub text: String,
    pub line_index: LineIndex,
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
        text: String,
        language: DocumentLanguage,
    ) -> Self {
        let line_index = LineIndex::new(&text);
        let data = match language {
            DocumentLanguage::Latex => {
                let root = latex::SyntaxNode::new_root(latex::parse(&text).green);

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
                let extras = context.extras;

                Box::new(LatexDocumentData {
                    green: root.green().into_owned(),
                    extras,
                })
                .into()
            }
            DocumentLanguage::Bibtex => {
                let root = bibtex::parse(&text).green;
                BibtexDocumentData { green: root }.into()
            }
            DocumentLanguage::BuildLog => DocumentData::BuildLog(build_log::parse(&text)),
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
