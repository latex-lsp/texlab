use crate::completion::latex::data::types::LatexComponentDatabase;
#[cfg(test)]
use crate::workspace::WorkspaceBuilder;
use crate::workspace::{Document, Workspace};
#[cfg(test)]
use lsp_types::*;
#[cfg(test)]
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FeatureRequest<T> {
    pub params: T,
    pub workspace: Arc<Workspace>,
    pub document: Arc<Document>,
    pub related_documents: Vec<Arc<Document>>,
    pub component_database: Arc<LatexComponentDatabase>,
}

impl<T> FeatureRequest<T> {
    pub fn new(
        params: T,
        workspace: Arc<Workspace>,
        document: Arc<Document>,
        component_database: Arc<LatexComponentDatabase>,
    ) -> Self {
        let related_documents = workspace.related_documents(&document.uri);
        FeatureRequest {
            params,
            workspace,
            document,
            related_documents,
            component_database,
        }
    }
}

#[macro_export]
macro_rules! concat_feature {
    ($request:expr, $($provider:tt), *) => {{
        let mut items = Vec::new();
        $(
            items.append(&mut await!($provider::execute($request)));
        )*
        items
    }};
}

#[macro_export]
macro_rules! choice_feature {
    ($request:expr, $provider:tt) => {
        await!($provider::execute($request))
    };
    ($request:expr, $provider:tt, $($providers:tt),+) => {{
        let value = await!($provider::execute($request));
        if value.is_some() {
            value
        } else {
            choice_feature!($request, $($providers),+)
        }
    }};
}

#[cfg(test)]
pub struct FeatureSpecFile {
    name: &'static str,
    text: &'static str,
}

#[cfg(test)]
pub struct FeatureSpec {
    pub files: Vec<FeatureSpecFile>,
    pub main_file: &'static str,
    pub position: Position,
    pub new_name: &'static str,
    pub component_database: LatexComponentDatabase,
}

#[cfg(test)]
impl FeatureSpec {
    pub fn file(name: &'static str, text: &'static str) -> FeatureSpecFile {
        FeatureSpecFile { name, text }
    }

    pub fn uri(name: &str) -> Uri {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(name);
        Uri::from_file_path(path).unwrap()
    }

    fn identifier(&self) -> TextDocumentIdentifier {
        let uri = Self::uri(self.main_file);
        TextDocumentIdentifier::new(uri)
    }

    fn workspace(&self) -> (Arc<Workspace>, Arc<Document>) {
        let mut builder = WorkspaceBuilder::new();
        for file in &self.files {
            builder.document(file.name, file.text);
        }
        let workspace = builder.workspace;
        let main_uri = Self::uri(self.main_file);
        let main_document = workspace.find(&main_uri).unwrap();
        (Arc::new(workspace), main_document)
    }
}

#[cfg(test)]
impl Into<FeatureRequest<TextDocumentPositionParams>> for FeatureSpec {
    fn into(self) -> FeatureRequest<TextDocumentPositionParams> {
        let params = TextDocumentPositionParams::new(self.identifier(), self.position);
        let (workspace, document) = self.workspace();
        FeatureRequest::new(
            params,
            workspace,
            document,
            Arc::new(self.component_database),
        )
    }
}

#[cfg(test)]
impl Into<FeatureRequest<CompletionParams>> for FeatureSpec {
    fn into(self) -> FeatureRequest<CompletionParams> {
        let params = CompletionParams {
            text_document: self.identifier(),
            position: self.position,
            context: None,
        };
        let (workspace, document) = self.workspace();
        FeatureRequest::new(
            params,
            workspace,
            document,
            Arc::new(self.component_database),
        )
    }
}

#[cfg(test)]
impl Into<FeatureRequest<FoldingRangeParams>> for FeatureSpec {
    fn into(self) -> FeatureRequest<FoldingRangeParams> {
        let params = FoldingRangeParams {
            text_document: self.identifier(),
        };
        let (workspace, document) = self.workspace();
        FeatureRequest::new(
            params,
            workspace,
            document,
            Arc::new(self.component_database),
        )
    }
}

#[cfg(test)]
impl Into<FeatureRequest<DocumentLinkParams>> for FeatureSpec {
    fn into(self) -> FeatureRequest<DocumentLinkParams> {
        let params = DocumentLinkParams {
            text_document: self.identifier(),
        };
        let (workspace, document) = self.workspace();
        FeatureRequest::new(
            params,
            workspace,
            document,
            Arc::new(self.component_database),
        )
    }
}

#[cfg(test)]
impl Into<FeatureRequest<ReferenceParams>> for FeatureSpec {
    fn into(self) -> FeatureRequest<ReferenceParams> {
        let params = ReferenceParams {
            text_document: self.identifier(),
            position: self.position,
            context: ReferenceContext {
                include_declaration: false,
            },
        };
        let (workspace, document) = self.workspace();
        FeatureRequest::new(
            params,
            workspace,
            document,
            Arc::new(self.component_database),
        )
    }
}

#[cfg(test)]
impl Into<FeatureRequest<RenameParams>> for FeatureSpec {
    fn into(self) -> FeatureRequest<RenameParams> {
        let params = RenameParams {
            text_document: self.identifier(),
            position: self.position,
            new_name: self.new_name.to_owned(),
        };
        let (workspace, document) = self.workspace();
        FeatureRequest::new(
            params,
            workspace,
            document,
            Arc::new(self.component_database),
        )
    }
}

#[cfg(test)]
#[macro_export]
macro_rules! test_feature {
    ($provider:tt, $spec: expr) => {{
        futures::executor::block_on($provider::execute(&$spec.into()))
    }};
}
