mod build;
#[cfg(feature = "completion")]
mod completion;
mod cursor;
mod definition;
mod folding;
mod formatting;
mod forward_search;
mod highlight;
mod hover;
mod link;
mod lsp_kinds;
mod reference;
mod rename;
mod symbol;

use std::sync::Arc;

use lsp_types::Url;

use crate::{Document, Workspace};

#[cfg(feature = "completion")]
pub use self::completion::{complete, CompletionItemData, COMPLETION_LIMIT};
pub use self::{
    build::{BuildEngine, BuildParams, BuildResult, BuildStatus},
    definition::goto_definition,
    folding::find_foldings,
    formatting::format_source_code,
    forward_search::{execute_forward_search, ForwardSearchResult, ForwardSearchStatus},
    highlight::find_document_highlights,
    hover::find_hover,
    link::find_document_links,
    reference::find_all_references,
    rename::{prepare_rename_all, rename_all},
    symbol::{find_document_symbols, find_workspace_symbols},
};

#[derive(Clone)]
pub struct FeatureRequest<P> {
    pub params: P,
    pub workspace: Workspace,
    pub uri: Arc<Url>,
}

impl<P> FeatureRequest<P> {
    pub fn main_document(&self) -> &Document {
        &self.workspace.documents_by_uri[&self.uri]
    }
}

#[cfg(test)]
mod testing {
    use std::{path::PathBuf, sync::Arc};

    use lsp_types::{
        ClientCapabilities, ClientInfo, CompletionParams, DocumentFormattingParams,
        DocumentHighlightParams, DocumentLinkParams, FoldingRangeParams, FormattingOptions,
        GotoDefinitionParams, HoverParams, PartialResultParams, Position, ReferenceContext,
        ReferenceParams, RenameParams, TextDocumentIdentifier, TextDocumentPositionParams,
        WorkDoneProgressParams,
    };
    use typed_builder::TypedBuilder;

    use crate::{distro::Resolver, DocumentLanguage, Environment, Options, Workspace};

    use super::*;

    #[derive(Debug, Clone, TypedBuilder)]
    pub struct FeatureTester<'a> {
        main: &'a str,

        files: Vec<(&'a str, &'a str)>,

        #[builder(default)]
        line: u32,

        #[builder(default)]
        character: u32,

        #[builder(default)]
        new_name: &'a str,

        #[builder(default)]
        include_declaration: bool,

        #[builder(default)]
        client_capabilities: ClientCapabilities,

        #[builder(default)]
        client_info: Option<ClientInfo>,

        #[builder(default)]
        resolver: Resolver,

        #[builder(default=std::env::temp_dir())]
        current_directory: PathBuf,

        #[builder(default, setter(strip_option))]
        root_directory: Option<PathBuf>,

        #[builder(default, setter(strip_option))]
        aux_directory: Option<PathBuf>,
    }

    impl<'a> FeatureTester<'a> {
        pub fn uri(&self, name: &str) -> Arc<Url> {
            let path = self.current_directory.join(name);
            Arc::new(Url::from_file_path(path).unwrap())
        }

        fn options(&self) -> Options {
            Options {
                aux_directory: self.aux_directory.clone(),
                root_directory: self.root_directory.clone(),
                ..Options::default()
            }
        }

        fn identifier(&self) -> TextDocumentIdentifier {
            let uri = self.uri(self.main);
            TextDocumentIdentifier::new(uri.as_ref().clone())
        }

        fn workspace(&self) -> Workspace {
            let mut workspace = Workspace::new(Environment {
                client_capabilities: Arc::new(self.client_capabilities.clone()),
                client_info: self.client_info.clone().map(Arc::new),
                options: Arc::new(self.options()),
                resolver: Arc::new(self.resolver.clone()),
                ..Environment::default()
            });

            for (name, source_code) in &self.files {
                let uri = self.uri(name);
                let path = uri.to_file_path().unwrap();
                let text = Arc::new(source_code.trim().to_string());
                let language = DocumentLanguage::by_path(&path).expect("unknown document language");
                let document = workspace.open(uri, text, language).unwrap();
                workspace.viewport.insert(document.uri);
            }

            workspace
        }

        fn request<P>(&self, params: P) -> FeatureRequest<P> {
            let workspace = self.workspace();
            let uri = self.uri(self.main);
            FeatureRequest {
                params,
                workspace: workspace.slice(&uri),
                uri,
            }
        }

        pub fn link(self) -> FeatureRequest<DocumentLinkParams> {
            let text_document = self.identifier();
            let params = DocumentLinkParams {
                text_document,
                work_done_progress_params: WorkDoneProgressParams::default(),
                partial_result_params: PartialResultParams::default(),
            };
            self.request(params)
        }

        pub fn folding(self) -> FeatureRequest<FoldingRangeParams> {
            let text_document = self.identifier();
            let params = FoldingRangeParams {
                text_document,
                work_done_progress_params: WorkDoneProgressParams::default(),
                partial_result_params: PartialResultParams::default(),
            };
            self.request(params)
        }

        pub fn reference(self) -> FeatureRequest<ReferenceParams> {
            let params = ReferenceParams {
                text_document_position: TextDocumentPositionParams::new(
                    self.identifier(),
                    Position::new(self.line, self.character),
                ),
                context: ReferenceContext {
                    include_declaration: self.include_declaration,
                },
                work_done_progress_params: WorkDoneProgressParams::default(),
                partial_result_params: PartialResultParams::default(),
            };
            self.request(params)
        }

        pub fn hover(self) -> FeatureRequest<HoverParams> {
            let params = HoverParams {
                text_document_position_params: TextDocumentPositionParams::new(
                    self.identifier(),
                    Position::new(self.line, self.character),
                ),
                work_done_progress_params: WorkDoneProgressParams::default(),
            };
            self.request(params)
        }

        pub fn completion(self) -> FeatureRequest<CompletionParams> {
            let params = CompletionParams {
                text_document_position: TextDocumentPositionParams::new(
                    self.identifier(),
                    Position::new(self.line, self.character),
                ),
                work_done_progress_params: WorkDoneProgressParams::default(),
                partial_result_params: PartialResultParams::default(),
                context: None,
            };

            self.request(params)
        }

        pub fn definition(self) -> FeatureRequest<GotoDefinitionParams> {
            let params = GotoDefinitionParams {
                text_document_position_params: TextDocumentPositionParams::new(
                    self.identifier(),
                    Position::new(self.line, self.character),
                ),
                work_done_progress_params: WorkDoneProgressParams::default(),
                partial_result_params: PartialResultParams::default(),
            };
            self.request(params)
        }

        pub fn rename(self) -> FeatureRequest<RenameParams> {
            let params = RenameParams {
                text_document_position: TextDocumentPositionParams::new(
                    self.identifier(),
                    Position::new(self.line, self.character),
                ),
                new_name: self.new_name.to_string(),
                work_done_progress_params: WorkDoneProgressParams::default(),
            };
            self.request(params)
        }

        pub fn formatting(self) -> FeatureRequest<DocumentFormattingParams> {
            let params = DocumentFormattingParams {
                text_document: self.identifier(),
                work_done_progress_params: WorkDoneProgressParams::default(),
                options: FormattingOptions::default(),
            };
            self.request(params)
        }

        pub fn highlight(self) -> FeatureRequest<DocumentHighlightParams> {
            let params = DocumentHighlightParams {
                text_document_position_params: TextDocumentPositionParams::new(
                    self.identifier(),
                    Position::new(self.line, self.character),
                ),
                work_done_progress_params: WorkDoneProgressParams::default(),
                partial_result_params: PartialResultParams::default(),
            };
            self.request(params)
        }
    }
}
