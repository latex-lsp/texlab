use std::time::SystemTime;
use texlab_distro::{Language, Resolver};
use texlab_protocol::*;
use texlab_syntax::{SyntaxTree, SyntaxTreeInput};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Document {
    pub uri: Uri,
    pub text: String,
    pub tree: SyntaxTree,
    pub modified: SystemTime,
}

impl Document {
    pub fn parse(
        uri: Uri,
        text: String,
        language: Language,
        options: &Options,
        resolver: &Resolver,
    ) -> Self {
        let input = SyntaxTreeInput {
            options,
            resolver,
            uri: &uri,
            text: &text,
            language,
        };
        let tree = SyntaxTree::parse(input);
        Self {
            uri,
            text,
            tree,
            modified: SystemTime::now(),
        }
    }

    pub fn is_file(&self) -> bool {
        self.uri.scheme() == "file"
    }
}
