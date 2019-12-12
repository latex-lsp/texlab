use std::time::SystemTime;
use texlab_distro::{Language, Resolver};
use texlab_protocol::*;
use texlab_syntax::{SyntaxTree, SyntaxTreeContext};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Document {
    pub uri: Uri,
    pub text: String,
    pub tree: SyntaxTree,
    pub modified: SystemTime,
}

impl Document {
    pub fn new(uri: Uri, text: String, tree: SyntaxTree) -> Self {
        Self {
            uri,
            text,
            tree,
            modified: SystemTime::now(),
        }
    }

    pub fn parse(resolver: &Resolver, uri: Uri, text: String, language: Language) -> Self {
        let context = SyntaxTreeContext {
            resolver,
            uri: &uri,
        };
        let tree = SyntaxTree::parse(context, &text, language);
        Self::new(uri, text, tree)
    }

    pub fn is_file(&self) -> bool {
        self.uri.scheme() == "file"
    }
}
