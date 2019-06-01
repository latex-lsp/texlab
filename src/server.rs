use crate::action::{Action, ActionMananger};
use crate::build::*;
use crate::client::LspClient;
use crate::completion::factory::CompletionItemData;
use crate::completion::CompletionProvider;
use crate::data::completion::LatexComponentDatabase;
use crate::data::component::ComponentDocumentation;
use crate::definition::DefinitionProvider;
use crate::diagnostics::{DiagnosticsManager, LatexLintOptions};
use crate::feature::{FeatureProvider, FeatureRequest};
use crate::folding::FoldingProvider;
use crate::formatting::bibtex;
use crate::formatting::bibtex::{BibtexFormattingOptions, BibtexFormattingParams};
use crate::highlight::HighlightProvider;
use crate::hover::HoverProvider;
use crate::link::LinkProvider;
use crate::reference::ReferenceProvider;
use crate::rename::RenameProvider;
use crate::request;
use crate::resolver;
use crate::resolver::TexResolver;
use crate::syntax::bibtex::BibtexDeclaration;
use crate::syntax::text::SyntaxNode;
use crate::syntax::SyntaxTree;
use crate::workspace::WorkspaceManager;
use futures::lock::Mutex;
use futures::prelude::*;
use futures_boxed::boxed;
use jsonrpc::server::Result;
use jsonrpc_derive::{jsonrpc_method, jsonrpc_server};
use log::*;
use lsp_types::*;
use serde::de::DeserializeOwned;
use std::borrow::Cow;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use walkdir::WalkDir;

pub struct LatexLspServer<C> {
    client: Arc<C>,
    workspace_manager: WorkspaceManager,
    action_manager: ActionMananger,
    diagnostics_manager: Mutex<DiagnosticsManager>,
    resolver: Mutex<Arc<TexResolver>>,
    completion_provider: CompletionProvider,
    definition_provider: DefinitionProvider,
    folding_provider: FoldingProvider,
    highlight_provider: HighlightProvider,
    hover_provider: HoverProvider,
    link_provider: LinkProvider,
    reference_provider: ReferenceProvider,
    rename_provider: RenameProvider,
}

#[jsonrpc_server]
impl<C: LspClient + Send + Sync> LatexLspServer<C> {
    pub fn new(client: Arc<C>) -> Self {
        LatexLspServer {
            client,
            workspace_manager: WorkspaceManager::default(),
            action_manager: ActionMananger::default(),
            diagnostics_manager: Mutex::new(DiagnosticsManager::default()),
            resolver: Mutex::new(Arc::new(TexResolver::default())),
            completion_provider: CompletionProvider::new(),
            definition_provider: DefinitionProvider::new(),
            folding_provider: FoldingProvider::new(),
            highlight_provider: HighlightProvider::new(),
            hover_provider: HoverProvider::new(),
            link_provider: LinkProvider::new(),
            reference_provider: ReferenceProvider::new(),
            rename_provider: RenameProvider::new(),
        }
    }

    #[jsonrpc_method("initialize", kind = "request")]
    pub async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        if let Some(Ok(path)) = params.root_uri.map(|x| x.to_file_path()) {
            for entry in WalkDir::new(path)
                .min_depth(1)
                .max_depth(4)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|x| x.file_type().is_file())
            {
                self.workspace_manager.load(&entry.path());
            }
        }

        let capabilities = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Options(
                TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(TextDocumentSyncKind::Full),
                    will_save: None,
                    will_save_wait_until: None,
                    save: Some(SaveOptions {
                        include_text: Some(false),
                    }),
                },
            )),
            hover_provider: Some(true),
            completion_provider: Some(CompletionOptions {
                resolve_provider: Some(true),
                trigger_characters: Some(vec!['\\', '{', '}', '@', '/']),
            }),
            signature_help_provider: None,
            definition_provider: Some(true),
            type_definition_provider: None,
            implementation_provider: None,
            references_provider: Some(true),
            document_highlight_provider: Some(true),
            document_symbol_provider: None,
            workspace_symbol_provider: None,
            code_action_provider: None,
            code_lens_provider: None,
            document_formatting_provider: Some(true),
            document_range_formatting_provider: None,
            document_on_type_formatting_provider: None,
            rename_provider: Some(RenameProviderCapability::Simple(true)),
            document_link_provider: Some(DocumentLinkOptions {
                resolve_provider: Some(false),
            }),
            color_provider: None,
            folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
            execute_command_provider: None,
            workspace: None,
            selection_range_provider: None,
        };

        Ok(InitializeResult { capabilities })
    }

    #[jsonrpc_method("initialized", kind = "notification")]
    pub fn initialized(&self, _params: InitializedParams) {
        self.action_manager.push(Action::RegisterCapabilities);
        self.action_manager.push(Action::LoadResolver);
        self.action_manager.push(Action::ResolveIncludes);
        self.action_manager.push(Action::PublishDiagnostics);
    }

    #[jsonrpc_method("shutdown", kind = "request")]
    pub async fn shutdown(&self, _params: ()) -> Result<()> {
        Ok(())
    }

    #[jsonrpc_method("exit", kind = "notification")]
    pub fn exit(&self, _params: ()) {}

    #[jsonrpc_method("workspace/didChangeWatchedFiles", kind = "notification")]
    pub fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        let workspace = self.workspace_manager.get();
        for change in params.changes {
            if change.uri.scheme() != "file" {
                continue;
            }

            let log_path = change.uri.to_file_path().unwrap();
            let name = log_path.to_string_lossy().into_owned();
            let tex_path = PathBuf::from(format!("{}tex", &name[0..name.len() - 3]));
            let tex_uri = Uri::from_file_path(tex_path).unwrap();
            if workspace.find(&tex_uri).is_some() {
                self.action_manager
                    .push(Action::ParseLog { tex_uri, log_path });
            }
        }
        self.action_manager.push(Action::PublishDiagnostics);
    }

    #[jsonrpc_method("textDocument/didOpen", kind = "notification")]
    pub fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.workspace_manager.add(params.text_document);
        self.action_manager.push(Action::ResolveIncludes);
        self.action_manager.push(Action::PublishDiagnostics);
    }

    #[jsonrpc_method("textDocument/didChange", kind = "notification")]
    pub fn did_change(&self, params: DidChangeTextDocumentParams) {
        for change in params.content_changes {
            let uri = params.text_document.uri.clone();
            self.workspace_manager.update(uri, change.text);
        }
        self.action_manager.push(Action::ResolveIncludes);
        self.action_manager.push(Action::PublishDiagnostics);
    }

    #[jsonrpc_method("textDocument/didSave", kind = "notification")]
    pub fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.action_manager
            .push(Action::RunLinter(params.text_document.uri.clone()));
        self.action_manager.push(Action::PublishDiagnostics);
        self.action_manager
            .push(Action::Build(params.text_document.uri));
    }

    #[jsonrpc_method("textDocument/didClose", kind = "notification")]
    pub fn did_close(&self, _params: DidCloseTextDocumentParams) {}

    #[jsonrpc_method("textDocument/completion", kind = "request")]
    pub async fn completion(&self, params: CompletionParams) -> Result<CompletionList> {
        let request = request!(self, params)?;
        let items = self.completion_provider.execute(&request).await;
        let all_includes = items.iter().all(|item| {
            item.kind == Some(CompletionItemKind::Folder)
                || item.kind == Some(CompletionItemKind::File)
        });
        Ok(CompletionList {
            is_incomplete: !all_includes,
            items,
        })
    }

    #[jsonrpc_method("completionItem/resolve", kind = "request")]
    pub async fn completion_resolve(&self, mut item: CompletionItem) -> Result<CompletionItem> {
        let data: CompletionItemData = serde_json::from_value(item.data.clone().unwrap()).unwrap();
        match data {
            CompletionItemData::Package | CompletionItemData::Class => {
                item.documentation = ComponentDocumentation::lookup(&item.label)
                    .await
                    .map(|documentation| Documentation::MarkupContent(documentation.content));
            }
            _ => {}
        };
        Ok(item)
    }

    #[jsonrpc_method("textDocument/hover", kind = "request")]
    pub async fn hover(&self, params: TextDocumentPositionParams) -> Result<Option<Hover>> {
        let request = request!(self, params)?;
        let hover = self.hover_provider.execute(&request).await;
        Ok(hover)
    }

    #[jsonrpc_method("textDocument/definition", kind = "request")]
    pub async fn definition(&self, params: TextDocumentPositionParams) -> Result<Vec<Location>> {
        let request = request!(self, params)?;
        let results = self.definition_provider.execute(&request).await;
        Ok(results)
    }

    #[jsonrpc_method("textDocument/references", kind = "request")]
    pub async fn references(&self, params: ReferenceParams) -> Result<Vec<Location>> {
        let request = request!(self, params)?;
        let results = self.reference_provider.execute(&request).await;
        Ok(results)
    }

    #[jsonrpc_method("textDocument/documentHighlight", kind = "request")]
    pub async fn document_highlight(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Vec<DocumentHighlight>> {
        let request = request!(self, params)?;
        let results = self.highlight_provider.execute(&request).await;
        Ok(results)
    }

    #[jsonrpc_method("textDocument/documentSymbol", kind = "request")]
    pub async fn document_symbol(
        &self,
        _params: DocumentSymbolParams,
    ) -> Result<Vec<DocumentSymbol>> {
        Ok(Vec::new())
    }

    #[jsonrpc_method("textDocument/documentLink", kind = "request")]
    pub async fn document_link(&self, params: DocumentLinkParams) -> Result<Vec<DocumentLink>> {
        let request = request!(self, params)?;
        let links = self.link_provider.execute(&request).await;
        Ok(links)
    }

    #[jsonrpc_method("textDocument/formatting", kind = "request")]
    pub async fn formatting(&self, params: DocumentFormattingParams) -> Result<Vec<TextEdit>> {
        let request = request!(self, params)?;
        let mut edits = Vec::new();
        if let SyntaxTree::Bibtex(tree) = &request.document.tree {
            let params = BibtexFormattingParams {
                tab_size: request.params.options.tab_size as usize,
                insert_spaces: request.params.options.insert_spaces,
                options: self
                    .configuration::<BibtexFormattingOptions>("bibtex.formatting")
                    .await,
            };

            for declaration in &tree.root.children {
                let should_format = match declaration {
                    BibtexDeclaration::Comment(_) => false,
                    BibtexDeclaration::Preamble(_) | BibtexDeclaration::String(_) => true,
                    BibtexDeclaration::Entry(entry) => !entry.is_comment(),
                };
                if should_format {
                    let text = bibtex::format_declaration(&declaration, &params);
                    edits.push(TextEdit::new(declaration.range(), Cow::from(text)));
                }
            }
        }
        Ok(edits)
    }

    #[jsonrpc_method("textDocument/rename", kind = "request")]
    pub async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let request = request!(self, params)?;
        let edit = self.rename_provider.execute(&request).await;
        Ok(edit)
    }

    #[jsonrpc_method("textDocument/foldingRange", kind = "request")]
    pub async fn folding_range(&self, params: FoldingRangeParams) -> Result<Vec<FoldingRange>> {
        let request = request!(self, params)?;
        let foldings = self.folding_provider.execute(&request).await;
        Ok(foldings)
    }

    #[jsonrpc_method("textDocument/build", kind = "request")]
    pub async fn build(&self, params: BuildParams) -> Result<BuildResult> {
        let request = request!(self, params)?;
        let options = self.configuration::<BuildOptions>("latex.build").await;
        let provider = BuildProvider::new(Arc::clone(&self.client), options);
        let result = provider.execute(&request).await;
        Ok(result)
    }

    async fn configuration<T>(&self, section: &'static str) -> T
    where
        T: DeserializeOwned + Default,
    {
        let params = ConfigurationParams {
            items: vec![ConfigurationItem {
                section: Some(Cow::from(section)),
                scope_uri: None,
            }],
        };

        match self.client.configuration(params).await {
            Ok(json) => match serde_json::from_value::<Vec<T>>(json) {
                Ok(config) => config.into_iter().next().unwrap(),
                Err(_) => {
                    warn!("Invalid configuration: {}", section);
                    T::default()
                }
            },
            Err(why) => {
                error!(
                    "Retrieving configuration for {} failed: {}",
                    section, why.message
                );
                T::default()
            }
        }
    }
}

impl<C: LspClient + Send + Sync> jsonrpc::ActionHandler for LatexLspServer<C> {
    #[boxed]
    async fn execute_actions(&self) {
        for action in self.action_manager.take() {
            match action {
                Action::RegisterCapabilities => {
                    let options = DidChangeWatchedFilesRegistrationOptions {
                        watchers: vec![FileSystemWatcher {
                            kind: Some(WatchKind::Create | WatchKind::Change),
                            glob_pattern: Cow::from("**/*.log"),
                        }],
                    };

                    if let Err(why) = self
                        .client
                        .register_capability(RegistrationParams {
                            registrations: vec![Registration {
                                id: Cow::from("build-log-watcher"),
                                method: Cow::from("workspace/didChangeWatchedFiles"),
                                register_options: Some(serde_json::to_value(options).unwrap()),
                            }],
                        })
                        .await
                    {
                        warn!(
                            "Client does not support dynamic capability registration: {}",
                            why.message
                        );
                    }
                }
                Action::LoadResolver => {
                    match TexResolver::load() {
                        Ok(res) => {
                            let mut resolver = self.resolver.lock().await;
                            *resolver = Arc::new(res);
                        }
                        Err(why) => {
                            let message = match why {
                                resolver::Error::KpsewhichNotFound => {
                                    "An error occurred while executing `kpsewhich`.\
                                     Please make sure that your distribution is in your PATH \
                                     environment variable and provides the `kpsewhich` tool."
                                }
                                resolver::Error::UnsupportedTexDistribution => {
                                    "Your TeX distribution is not supported."
                                }
                                resolver::Error::CorruptFileDatabase => {
                                    "The file database of your TeX distribution seems \
                                     to be corrupt. Please rebuild it and try again."
                                }
                            };

                            let params = ShowMessageParams {
                                message: Cow::from(message),
                                typ: MessageType::Error,
                            };

                            self.client.show_message(params).await;
                        }
                    };
                }
                Action::ResolveIncludes => {
                    let workspace = self.workspace_manager.get();
                    workspace
                        .unresolved_includes()
                        .iter()
                        .for_each(|path| self.workspace_manager.load(&path));
                }
                Action::PublishDiagnostics => {
                    let workspace = self.workspace_manager.get();
                    let diagnostics_manager = self.diagnostics_manager.lock().await;
                    for document in &workspace.documents {
                        self.client
                            .publish_diagnostics(PublishDiagnosticsParams {
                                uri: document.uri.clone(),
                                diagnostics: diagnostics_manager.get(&document),
                            })
                            .await;
                    }
                }
                Action::RunLinter(uri) => {
                    let config: LatexLintOptions = self.configuration("latex.lint").await;
                    if config.on_save {
                        let mut diagnostics_manager = self.diagnostics_manager.lock().await;
                        diagnostics_manager.latex.update(&uri);
                    }
                }
                Action::ParseLog { tex_uri, log_path } => {
                    if let Ok(log) = fs::read_to_string(&log_path) {
                        let mut diagnostics_manager = self.diagnostics_manager.lock().await;
                        diagnostics_manager.build.update(&tex_uri, &log);
                    }
                }
                Action::Build(uri) => {
                    let config: BuildOptions = self.configuration("latex.build").await;
                    if config.on_save {
                        let text_document = TextDocumentIdentifier::new(uri);
                        self.build(BuildParams { text_document }).await.unwrap();
                    }
                }
            }
        }
    }
}

#[macro_export]
macro_rules! request {
    ($server:expr, $params:expr) => {{
        let workspace = $server.workspace_manager.get();
        let resolver = $server.resolver.lock().await;

        if let Some(document) = workspace.find(&$params.text_document.uri) {
            Ok(FeatureRequest::new(
                $params,
                workspace,
                document,
                Arc::clone(&resolver),
                Arc::new(LatexComponentDatabase::default()),
            ))
        } else {
            let msg = format!("Unknown document: {}", $params.text_document.uri);
            Err(msg)
        }
    }};
}
