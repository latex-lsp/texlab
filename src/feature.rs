use crate::data::completion::LatexComponentDatabase;
use crate::tex::resolver::TexResolver;
#[cfg(test)]
use crate::workspace::WorkspaceBuilder;
use crate::workspace::{Document, Workspace};
use futures_boxed::boxed;
#[cfg(test)]
use lsp_types::*;
#[cfg(test)]
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FeatureRequest<P> {
    pub params: P,
    pub workspace: Arc<Workspace>,
    pub document: Arc<Document>,
    pub related_documents: Vec<Arc<Document>>,
    pub resolver: Arc<TexResolver>,
    pub component_database: Arc<LatexComponentDatabase>,
}

impl<P> FeatureRequest<P> {
    pub fn new(
        params: P,
        workspace: Arc<Workspace>,
        document: Arc<Document>,
        resolver: Arc<TexResolver>,
        component_database: Arc<LatexComponentDatabase>,
    ) -> Self {
        let related_documents = workspace.related_documents(&document.uri);
        Self {
            params,
            workspace,
            document,
            related_documents,
            resolver,
            component_database,
        }
    }
}

pub trait FeatureProvider {
    type Params;
    type Output;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output;
}

type ListProvider<P, O> = Box<FeatureProvider<Params = P, Output = Vec<O>> + Send + Sync>;

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

type OptionProvider<P, O> = Box<FeatureProvider<Params = P, Output = Option<O>> + Send + Sync>;

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

#[cfg(test)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FeatureSpecFile {
    name: &'static str,
    text: &'static str,
}

#[cfg(test)]
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct FeatureSpec {
    pub files: Vec<FeatureSpecFile>,
    pub main_file: &'static str,
    pub position: Position,
    pub new_name: &'static str,
    pub resolver: TexResolver,
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
            Arc::new(self.resolver),
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
            Arc::new(self.resolver),
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
            Arc::new(self.resolver),
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
            Arc::new(self.resolver),
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
            Arc::new(self.resolver),
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
            Arc::new(self.resolver),
            Arc::new(self.component_database),
        )
    }
}

#[cfg(test)]
pub fn test_feature<F, P, O, S>(provider: F, spec: S) -> O
where
    F: FeatureProvider<Params = P, Output = O>,
    S: Into<FeatureRequest<P>>,
{
    futures::executor::block_on(provider.execute(&spec.into()))
}
