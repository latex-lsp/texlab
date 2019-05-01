use crate::workspace::{Document, Workspace};
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FeatureRequest<T> {
    pub params: T,
    pub workspace: Workspace,
    pub document: Arc<Document>,
    pub related_documents: Vec<Arc<Document>>,
}

impl<T> FeatureRequest<T> {
    pub fn new(params: T, workspace: Workspace, document: Arc<Document>) -> Self {
        let related_documents = workspace.related_documents(&document.uri);
        FeatureRequest {
            params,
            workspace,
            document,
            related_documents,
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

#[cfg(test)]
use lsp_types::*;

#[cfg(test)]
pub struct FeatureTester {
    workspace: Workspace,
    document: Arc<Document>,
    document_id: TextDocumentIdentifier,
    position: Position,
    new_name: String,
}

#[cfg(test)]
impl FeatureTester {
    pub fn new(workspace: Workspace, uri: Url, line: u64, character: u64, new_name: &str) -> Self {
        let document = workspace.find(&uri).unwrap();
        FeatureTester {
            workspace,
            document,
            document_id: TextDocumentIdentifier::new(uri),
            position: Position::new(line, character),
            new_name: new_name.to_owned(),
        }
    }
}

#[cfg(test)]
impl Into<FeatureRequest<TextDocumentPositionParams>> for FeatureTester {
    fn into(self) -> FeatureRequest<TextDocumentPositionParams> {
        let params = TextDocumentPositionParams::new(self.document_id, self.position);
        FeatureRequest::new(params, self.workspace, self.document)
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
        FeatureRequest::new(params, self.workspace, self.document)
    }
}

#[cfg(test)]
impl Into<FeatureRequest<FoldingRangeParams>> for FeatureTester {
    fn into(self) -> FeatureRequest<FoldingRangeParams> {
        let params = FoldingRangeParams {
            text_document: self.document_id,
        };
        FeatureRequest::new(params, self.workspace, self.document)
    }
}

#[cfg(test)]
impl Into<FeatureRequest<DocumentLinkParams>> for FeatureTester {
    fn into(self) -> FeatureRequest<DocumentLinkParams> {
        let params = DocumentLinkParams {
            text_document: self.document_id,
        };
        FeatureRequest::new(params, self.workspace, self.document)
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
        FeatureRequest::new(params, self.workspace, self.document)
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
        FeatureRequest::new(params, self.workspace, self.document)
    }
}
