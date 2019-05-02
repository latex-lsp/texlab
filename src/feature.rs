use crate::completion::latex::data::types::LatexComponentDatabase;
use crate::workspace::{Document, Workspace, WorkspaceBuilder};
#[cfg(test)]
use lsp_types::*;
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

    pub fn uri(name: &str) -> Url {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(name);
        Url::from_file_path(path).unwrap()
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
#[macro_export]
macro_rules! test_feature {
    ($provider:tt, $spec: expr) => {{
        futures::executor::block_on($provider::execute(&$spec.into()))
    }};
}

#[cfg(test)]
pub struct FeatureTester {
    workspace: Arc<Workspace>,
    document: Arc<Document>,
    document_id: TextDocumentIdentifier,
    position: Position,
    new_name: String,
    component_database: Arc<LatexComponentDatabase>,
}

#[cfg(test)]
impl FeatureTester {
    pub fn new(workspace: Workspace, uri: Url, line: u64, character: u64, new_name: &str) -> Self {
        let document = workspace.find(&uri).unwrap();
        FeatureTester {
            workspace: Arc::new(workspace),
            document,
            document_id: TextDocumentIdentifier::new(uri),
            position: Position::new(line, character),
            new_name: new_name.to_owned(),
            component_database: Arc::new(LatexComponentDatabase::default()),
        }
    }
}

#[cfg(test)]
impl Into<FeatureRequest<TextDocumentPositionParams>> for FeatureTester {
    fn into(self) -> FeatureRequest<TextDocumentPositionParams> {
        let params = TextDocumentPositionParams::new(self.document_id, self.position);
        FeatureRequest::new(
            params,
            self.workspace,
            self.document,
            self.component_database,
        )
    }
}

#[cfg(test)]
impl Into<FeatureRequest<CompletionParams>> for FeatureTester {
    fn into(self) -> FeatureRequest<CompletionParams> {
        let params = CompletionParams {
            text_document: self.document_id,
            position: self.position,
            context: None,
        };
        FeatureRequest::new(
            params,
            self.workspace,
            self.document,
            self.component_database,
        )
    }
}

#[cfg(test)]
impl Into<FeatureRequest<FoldingRangeParams>> for FeatureTester {
    fn into(self) -> FeatureRequest<FoldingRangeParams> {
        let params = FoldingRangeParams {
            text_document: self.document_id,
        };
        FeatureRequest::new(
            params,
            self.workspace,
            self.document,
            self.component_database,
        )
    }
}

#[cfg(test)]
impl Into<FeatureRequest<DocumentLinkParams>> for FeatureTester {
    fn into(self) -> FeatureRequest<DocumentLinkParams> {
        let params = DocumentLinkParams {
            text_document: self.document_id,
        };
        FeatureRequest::new(
            params,
            self.workspace,
            self.document,
            self.component_database,
        )
    }
}

#[cfg(test)]
impl Into<FeatureRequest<ReferenceParams>> for FeatureTester {
    fn into(self) -> FeatureRequest<ReferenceParams> {
        let params = ReferenceParams {
            text_document: self.document_id,
            position: self.position,
            context: ReferenceContext {
                include_declaration: false,
            },
        };
        FeatureRequest::new(
            params,
            self.workspace,
            self.document,
            self.component_database,
        )
    }
}

#[cfg(test)]
impl Into<FeatureRequest<RenameParams>> for FeatureTester {
    fn into(self) -> FeatureRequest<RenameParams> {
        let params = RenameParams {
            text_document: self.document_id,
            position: self.position,
            new_name: self.new_name,
        };
        FeatureRequest::new(
            params,
            self.workspace,
            self.document,
            self.component_database,
        )
    }
}
