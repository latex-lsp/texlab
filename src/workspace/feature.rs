use super::{Document, DocumentView, Workspace, WorkspaceBuilder};
use futures_boxed::boxed;
use lsp_types::*;
use std::sync::Arc;

#[derive(Debug, PartialEq, Clone)]
pub struct FeatureRequest<P> {
    pub params: P,
    pub view: DocumentView,
    pub client_capabilities: Arc<ClientCapabilities>,
}

impl<P> FeatureRequest<P> {
    pub fn workspace(&self) -> &Workspace {
        &self.view.workspace
    }

    pub fn document(&self) -> &Document {
        &self.view.document
    }

    pub fn related_documents(&self) -> &[Arc<Document>] {
        &self.view.related_documents
    }
}

pub trait FeatureProvider {
    type Params;
    type Output;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output;
}

type ListProvider<P, O> = Box<dyn FeatureProvider<Params = P, Output = Vec<O>> + Send + Sync>;

#[derive(Default)]
pub struct ConcatProvider<P, O> {
    providers: Vec<ListProvider<P, O>>,
}

impl<P, O> ConcatProvider<P, O> {
    pub fn new(providers: Vec<ListProvider<P, O>>) -> Self {
        Self { providers }
    }
}

impl<P, O> FeatureProvider for ConcatProvider<P, O>
where
    P: Send + Sync,
    O: Send + Sync,
{
    type Params = P;
    type Output = Vec<O>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<P>) -> Vec<O> {
        let mut items = Vec::new();
        for provider in &self.providers {
            items.append(&mut provider.execute(request).await);
        }
        items
    }
}

type OptionProvider<P, O> = Box<dyn FeatureProvider<Params = P, Output = Option<O>> + Send + Sync>;

#[derive(Default)]
pub struct ChoiceProvider<P, O> {
    providers: Vec<OptionProvider<P, O>>,
}

impl<P, O> ChoiceProvider<P, O> {
    pub fn new(providers: Vec<OptionProvider<P, O>>) -> Self {
        Self { providers }
    }
}

impl<P, O> FeatureProvider for ChoiceProvider<P, O>
where
    P: Send + Sync,
    O: Send + Sync,
{
    type Params = P;
    type Output = Option<O>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<P>) -> Option<O> {
        for provider in &self.providers {
            let item = provider.execute(request).await;
            if item.is_some() {
                return item;
            }
        }
        None
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FeatureSpecFile {
    name: &'static str,
    text: &'static str,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct FeatureSpec {
    pub files: Vec<FeatureSpecFile>,
    pub main_file: &'static str,
    pub position: Position,
    pub new_name: &'static str,
    pub include_declaration: bool,
    pub client_capabilities: ClientCapabilities,
}

impl FeatureSpec {
    pub fn file(name: &'static str, text: &'static str) -> FeatureSpecFile {
        FeatureSpecFile { name, text }
    }

    pub fn uri(name: &str) -> Uri {
        let path = std::env::temp_dir().join(name);
        Uri::from_file_path(path).unwrap()
    }

    fn identifier(&self) -> TextDocumentIdentifier {
        let uri = Self::uri(self.main_file);
        TextDocumentIdentifier::new(uri)
    }

    fn view(&self) -> DocumentView {
        let mut builder = WorkspaceBuilder::new();
        for file in &self.files {
            builder.document(file.name, file.text);
        }
        let workspace = builder.workspace;
        let main_uri = Self::uri(self.main_file);
        let main_document = workspace.find(&main_uri).unwrap();
        DocumentView::new(Arc::new(workspace), main_document)
    }

    fn request<T>(self, params: T) -> FeatureRequest<T> {
        FeatureRequest {
            params,
            view: self.view(),
            client_capabilities: Arc::new(self.client_capabilities),
        }
    }
}

impl Into<FeatureRequest<TextDocumentPositionParams>> for FeatureSpec {
    fn into(self) -> FeatureRequest<TextDocumentPositionParams> {
        let params = TextDocumentPositionParams::new(self.identifier(), self.position);
        self.request(params)
    }
}

impl Into<FeatureRequest<CompletionParams>> for FeatureSpec {
    fn into(self) -> FeatureRequest<CompletionParams> {
        let params = CompletionParams {
            text_document: self.identifier(),
            position: self.position,
            context: None,
        };
        self.request(params)
    }
}

impl Into<FeatureRequest<FoldingRangeParams>> for FeatureSpec {
    fn into(self) -> FeatureRequest<FoldingRangeParams> {
        let params = FoldingRangeParams {
            text_document: self.identifier(),
        };
        self.request(params)
    }
}

impl Into<FeatureRequest<DocumentLinkParams>> for FeatureSpec {
    fn into(self) -> FeatureRequest<DocumentLinkParams> {
        let params = DocumentLinkParams {
            text_document: self.identifier(),
        };
        self.request(params)
    }
}

impl Into<FeatureRequest<ReferenceParams>> for FeatureSpec {
    fn into(self) -> FeatureRequest<ReferenceParams> {
        let params = ReferenceParams {
            text_document: self.identifier(),
            position: self.position,
            context: ReferenceContext {
                include_declaration: self.include_declaration,
            },
        };
        self.request(params)
    }
}

impl Into<FeatureRequest<RenameParams>> for FeatureSpec {
    fn into(self) -> FeatureRequest<RenameParams> {
        let params = RenameParams {
            text_document: self.identifier(),
            position: self.position,
            new_name: self.new_name.to_owned(),
        };
        self.request(params)
    }
}

pub fn test_feature<F, P, O, S>(provider: F, spec: S) -> O
where
    F: FeatureProvider<Params = P, Output = O>,
    S: Into<FeatureRequest<P>>,
{
    futures::executor::block_on(provider.execute(&spec.into()))
}
