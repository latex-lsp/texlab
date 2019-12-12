use crate::action::{Action, ActionManager, LintReason};
use crate::build::*;
use texlab_citeproc::render_citation;
use crate::definition::DefinitionProvider;
use crate::diagnostics::DiagnosticsManager;
use crate::folding::FoldingProvider;
use crate::forward_search;
use crate::highlight::HighlightProvider;
use crate::hover::HoverProvider;
use crate::link::LinkProvider;
use crate::reference::ReferenceProvider;
use crate::rename::{PrepareRenameProvider, RenameProvider};
use crate::symbol::{self, SymbolProvider};
use futures::lock::Mutex;
use futures_boxed::boxed;
use jsonrpc::server::{Middleware, Result};
use jsonrpc_derive::{jsonrpc_method, jsonrpc_server};
use log::*;
use once_cell::sync::{Lazy, OnceCell};
use serde::de::DeserializeOwned;
use std::ffi::OsStr;
use std::fs;
use std::future::Future;
use std::sync::Arc;
use texlab_completion::{CompletionItemData, CompletionProvider};
use texlab_distro::{Distribution, DistributionKind, Language};
use texlab_protocol::*;
use texlab_syntax::*;
use texlab_workspace::*;
use walkdir::WalkDir;

pub struct LatexLspServer<C> {
    client: Arc<C>,
    client_capabilities: OnceCell<Arc<ClientCapabilities>>,
    distribution: Arc<Box<dyn Distribution>>,
    build_manager: BuildManager<C>,
    workspace_manager: WorkspaceManager,
    action_manager: ActionManager,
    diagnostics_manager: Mutex<DiagnosticsManager>,
    completion_provider: CompletionProvider,
    definition_provider: DefinitionProvider,
    folding_provider: FoldingProvider,
    highlight_provider: HighlightProvider,
    symbol_provider: SymbolProvider,
    hover_provider: HoverProvider,
    link_provider: LinkProvider,
    reference_provider: ReferenceProvider,
    prepare_rename_provider: PrepareRenameProvider,
    rename_provider: RenameProvider,
}

#[jsonrpc_server]
impl<C: LspClient + Send + Sync + 'static> LatexLspServer<C> {
    pub fn new(client: Arc<C>, distribution: Arc<Box<dyn Distribution>>) -> Self {
        Self {
            client: Arc::clone(&client),
            client_capabilities: OnceCell::new(),
            distribution: Arc::clone(&distribution),
            build_manager: BuildManager::new(client),
            workspace_manager: WorkspaceManager::new(distribution),
            action_manager: ActionManager::default(),
            diagnostics_manager: Mutex::new(DiagnosticsManager::default()),
            completion_provider: CompletionProvider::new(),
            definition_provider: DefinitionProvider::new(),
            folding_provider: FoldingProvider::new(),
            highlight_provider: HighlightProvider::new(),
            symbol_provider: SymbolProvider::new(),
            hover_provider: HoverProvider::new(),
            link_provider: LinkProvider::new(),
            reference_provider: ReferenceProvider::new(),
            prepare_rename_provider: PrepareRenameProvider::new(),
            rename_provider: RenameProvider::new(),
        }
    }

    pub async fn execute<'a, T, A>(&'a self, action: A) -> T
    where
        A: FnOnce(&'a Self) -> T,
    {
        self.before_message().await;
        let result = action(&self);
        self.after_message().await;
        result
    }

    pub async fn execute_async<'a, T, F, A>(&'a self, action: A) -> T
    where
        F: Future<Output = T>,
        A: FnOnce(&'a Self) -> F,
    {
        self.before_message().await;
        let result = action(&self).await;
        self.after_message().await;
        result
    }

    #[jsonrpc_method("initialize", kind = "request")]
    pub async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        self.client_capabilities
            .set(Arc::new(params.capabilities))
            .unwrap();
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
                trigger_characters: Some(vec![
                    "\\".to_owned(),
                    "{".to_owned(),
                    "}".to_owned(),
                    "@".to_owned(),
                    "/".to_owned(),
                    " ".to_owned(),
                ]),
            }),
            signature_help_provider: None,
            definition_provider: Some(true),
            type_definition_provider: None,
            implementation_provider: None,
            references_provider: Some(true),
            document_highlight_provider: Some(true),
            document_symbol_provider: Some(true),
            workspace_symbol_provider: Some(true),
            code_action_provider: None,
            code_lens_provider: None,
            document_formatting_provider: Some(true),
            document_range_formatting_provider: None,
            document_on_type_formatting_provider: None,
            rename_provider: Some(RenameProviderCapability::Options(RenameOptions {
                prepare_provider: Some(true),
            })),
            document_link_provider: Some(DocumentLinkOptions {
                resolve_provider: Some(false),
            }),
            color_provider: None,
            folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
            execute_command_provider: None,
            workspace: None,
            selection_range_provider: None,
        };

        Lazy::force(&COMPLETION_DATABASE);
        Ok(InitializeResult { capabilities })
    }

    #[jsonrpc_method("initialized", kind = "notification")]
    pub fn initialized(&self, _params: InitializedParams) {
        self.action_manager.push(Action::PublishDiagnostics);
        self.action_manager.push(Action::LoadDistribution);
    }

    #[jsonrpc_method("shutdown", kind = "request")]
    pub async fn shutdown(&self, _params: ()) -> Result<()> {
        Ok(())
    }

    #[jsonrpc_method("exit", kind = "notification")]
    pub fn exit(&self, _params: ()) {}

    #[jsonrpc_method("$/cancelRequest", kind = "notification")]
    pub fn cancel_request(&self, _params: CancelParams) {}

    #[jsonrpc_method("textDocument/didOpen", kind = "notification")]
    pub fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        self.workspace_manager.add(params.text_document);
        self.action_manager
            .push(Action::DetectRoot(uri.clone().into()));
        self.action_manager
            .push(Action::RunLinter(Uri::from(uri), LintReason::Save));
        self.action_manager.push(Action::PublishDiagnostics);
    }

    #[jsonrpc_method("textDocument/didChange", kind = "notification")]
    pub fn did_change(&self, params: DidChangeTextDocumentParams) {
        for change in params.content_changes {
            let uri = params.text_document.uri.clone();
            self.workspace_manager.update(uri.into(), change.text);
        }
        self.action_manager.push(Action::RunLinter(
            params.text_document.uri.into(),
            LintReason::Change,
        ));
        self.action_manager.push(Action::PublishDiagnostics);
    }

    #[jsonrpc_method("textDocument/didSave", kind = "notification")]
    pub fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.action_manager.push(Action::RunLinter(
            params.text_document.uri.clone().into(),
            LintReason::Save,
        ));
        self.action_manager.push(Action::PublishDiagnostics);
        self.action_manager
            .push(Action::Build(params.text_document.uri.into()));
    }

    #[jsonrpc_method("textDocument/didClose", kind = "notification")]
    pub fn did_close(&self, _params: DidCloseTextDocumentParams) {}

    #[jsonrpc_method("window/workDoneProgress/cancel", kind = "notification")]
    pub fn work_done_progress_cancel(&self, params: WorkDoneProgressCancelParams) {
        self.action_manager.push(Action::CancelBuild(params.token));
    }

    #[jsonrpc_method("textDocument/completion", kind = "request")]
    pub async fn completion(&self, params: CompletionParams) -> Result<CompletionList> {
        let request = self.make_feature_request(params.text_document_position.as_uri(), params)?;
        let items = self.completion_provider.execute(&request).await;
        Ok(CompletionList {
            is_incomplete: true,
            items,
        })
    }

    #[jsonrpc_method("completionItem/resolve", kind = "request")]
    pub async fn completion_resolve(&self, mut item: CompletionItem) -> Result<CompletionItem> {
        let data: CompletionItemData = serde_json::from_value(item.data.clone().unwrap()).unwrap();
        match data {
            CompletionItemData::Package | CompletionItemData::Class => {
                item.documentation = COMPLETION_DATABASE
                    .documentation(&item.label)
                    .map(Documentation::MarkupContent);
            }
            CompletionItemData::Citation { uri, key } => {
                let workspace = self.workspace_manager.get();
                if let Some(document) = workspace.find(&uri) {
                    if let SyntaxTree::Bibtex(tree) = &document.tree {
                        let markup = render_citation(&tree, &key);
                        item.documentation = markup.map(Documentation::MarkupContent);
                    }
                }
            }
            _ => {}
        };
        Ok(item)
    }

    #[jsonrpc_method("textDocument/hover", kind = "request")]
    pub async fn hover(&self, params: TextDocumentPositionParams) -> Result<Option<Hover>> {
        let request = self.make_feature_request(params.text_document.as_uri(), params)?;
        let hover = self.hover_provider.execute(&request).await;
        Ok(hover)
    }

    #[jsonrpc_method("textDocument/definition", kind = "request")]
    pub async fn definition(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<DefinitionResponse> {
        let request = self.make_feature_request(params.text_document.as_uri(), params)?;
        let results = self.definition_provider.execute(&request).await;
        let response = if request.client_capabilities.has_definition_link_support() {
            DefinitionResponse::LocationLinks(results)
        } else {
            DefinitionResponse::Locations(
                results
                    .into_iter()
                    .map(|link| Location::new(link.target_uri, link.target_selection_range))
                    .collect(),
            )
        };

        Ok(response)
    }

    #[jsonrpc_method("textDocument/references", kind = "request")]
    pub async fn references(&self, params: ReferenceParams) -> Result<Vec<Location>> {
        let request = self.make_feature_request(params.text_document_position.as_uri(), params)?;
        let results = self.reference_provider.execute(&request).await;
        Ok(results)
    }

    #[jsonrpc_method("textDocument/documentHighlight", kind = "request")]
    pub async fn document_highlight(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Vec<DocumentHighlight>> {
        let request = self.make_feature_request(params.text_document.as_uri(), params)?;
        let results = self.highlight_provider.execute(&request).await;
        Ok(results)
    }

    #[jsonrpc_method("workspace/symbol", kind = "request")]
    pub async fn workspace_symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Vec<SymbolInformation>> {
        let distribution = Arc::clone(&self.distribution);
        let client_capabilities = Arc::clone(&self.client_capabilities.get().unwrap());
        let workspace = self.workspace_manager.get();
        let symbols =
            symbol::workspace_symbols(distribution, client_capabilities, workspace, &params).await;
        Ok(symbols)
    }

    #[jsonrpc_method("textDocument/documentSymbol", kind = "request")]
    pub async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<DocumentSymbolResponse> {
        let request = self.make_feature_request(params.text_document.as_uri(), params)?;
        let symbols = self.symbol_provider.execute(&request).await;
        let response = symbol::document_symbols(
            &self.client_capabilities.get().unwrap(),
            &request.view.workspace,
            &request.document().uri,
            symbols.into_iter().map(Into::into).collect(),
        );
        Ok(response)
    }

    #[jsonrpc_method("textDocument/documentLink", kind = "request")]
    pub async fn document_link(&self, params: DocumentLinkParams) -> Result<Vec<DocumentLink>> {
        let request = self.make_feature_request(params.text_document.as_uri(), params)?;
        let links = self.link_provider.execute(&request).await;
        Ok(links)
    }

    #[jsonrpc_method("textDocument/formatting", kind = "request")]
    pub async fn formatting(&self, params: DocumentFormattingParams) -> Result<Vec<TextEdit>> {
        let request = self.make_feature_request(params.text_document.as_uri(), params)?;
        let mut edits = Vec::new();
        if let SyntaxTree::Bibtex(tree) = &request.document().tree {
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
                    let text = format_declaration(&declaration, &params);
                    edits.push(TextEdit::new(declaration.range(), text));
                }
            }
        }
        Ok(edits)
    }

    #[jsonrpc_method("textDocument/prepareRename", kind = "request")]
    pub async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<Range>> {
        let request = self.make_feature_request(params.as_uri(), params)?;
        let range = self.prepare_rename_provider.execute(&request).await;
        Ok(range)
    }

    #[jsonrpc_method("textDocument/rename", kind = "request")]
    pub async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let request = self.make_feature_request(params.text_document_position.as_uri(), params)?;
        let edit = self.rename_provider.execute(&request).await;
        Ok(edit)
    }

    #[jsonrpc_method("textDocument/foldingRange", kind = "request")]
    pub async fn folding_range(&self, params: FoldingRangeParams) -> Result<Vec<FoldingRange>> {
        let request = self.make_feature_request(params.text_document.as_uri(), params)?;
        let foldings = self.folding_provider.execute(&request).await;
        Ok(foldings)
    }

    #[jsonrpc_method("textDocument/build", kind = "request")]
    pub async fn build(&self, params: BuildParams) -> Result<BuildResult> {
        let request = self.make_feature_request(params.text_document.as_uri(), params)?;
        let options = self.configuration::<LatexBuildOptions>("latex.build").await;
        let result = self.build_manager.build(request, options).await;
        Ok(result)
    }

    #[jsonrpc_method("textDocument/forwardSearch", kind = "request")]
    pub async fn forward_search(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<ForwardSearchResult> {
        let request = self.make_feature_request(params.text_document.as_uri(), params)?;
        let options = self
            .configuration::<LatexForwardSearchOptions>("latex.forwardSearch")
            .await;

        match request.document().uri.to_file_path() {
            Ok(tex_file) => {
                let parent = request
                    .workspace()
                    .find_parent(&request.document().uri)
                    .unwrap_or(request.view.document);
                let parent = parent.uri.to_file_path().unwrap();
                forward_search::search(&tex_file, &parent, request.params.position.line, options)
                    .await
                    .ok_or_else(|| "Unable to execute forward search".into())
            }
            Err(()) => Ok(ForwardSearchResult {
                status: ForwardSearchStatus::Failure,
            }),
        }
    }

    async fn configuration<T>(&self, section: &'static str) -> T
    where
        T: DeserializeOwned + Default,
    {
        if !self
            .client_capabilities
            .get()
            .and_then(|cap| cap.workspace.as_ref())
            .and_then(|cap| cap.configuration)
            .unwrap_or(false)
        {
            return T::default();
        }

        let params = ConfigurationParams {
            items: vec![ConfigurationItem {
                section: Some(section.into()),
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

    fn make_feature_request<P>(&self, uri: Uri, params: P) -> Result<FeatureRequest<P>> {
        let workspace = self.workspace_manager.get();
        let client_capabilities = self
            .client_capabilities
            .get()
            .expect("Failed to retrieve client capabilities");

        if let Some(document) = workspace.find(&uri) {
            Ok(FeatureRequest {
                params,
                view: DocumentView::new(workspace, document),
                client_capabilities: Arc::clone(&client_capabilities),
                distribution: Arc::clone(&self.distribution),
            })
        } else {
            let msg = format!("Unknown document: {}", uri);
            Err(msg)
        }
    }

    async fn detect_children(&self) {
        loop {
            let mut changed = false;

            let workspace = self.workspace_manager.get();
            for path in workspace.unresolved_includes() {
                if path.exists() {
                    changed |= self.workspace_manager.load(&path).is_ok();
                }
            }

            if !changed {
                break;
            }
        }
    }

    fn update_document(&self, document: &Document) -> std::result::Result<(), LoadError> {
        if document.uri.scheme() != "file" {
            return Ok(());
        }

        let path = document.uri.to_file_path().unwrap();
        let data = fs::metadata(&path).map_err(LoadError::IO)?;
        if data.modified().map_err(LoadError::IO)? > document.modified {
            self.workspace_manager.load(&path)
        } else {
            Ok(())
        }
    }

    async fn update_build_diagnostics(&self) {
        let workspace = self.workspace_manager.get();
        let mut diagnostics_manager = self.diagnostics_manager.lock().await;
        for document in &workspace.documents {
            if document.uri.scheme() != "file" {
                continue;
            }

            if let SyntaxTree::Latex(tree) = &document.tree {
                if tree.env.is_standalone {
                    match diagnostics_manager.build.update(&document.uri) {
                        Ok(true) => self.action_manager.push(Action::PublishDiagnostics),
                        Ok(false) => (),
                        Err(why) => warn!(
                            "Unable to read log file ({}): {}",
                            why,
                            document.uri.as_str()
                        ),
                    }
                }
            }
        }
    }
}

impl<C: LspClient + Send + Sync + 'static> Middleware for LatexLspServer<C> {
    #[boxed]
    async fn before_message(&self) {
        self.detect_children().await;

        let workspace = self.workspace_manager.get();
        for document in &workspace.documents {
            let _ = self.update_document(document);
        }
    }

    #[boxed]
    async fn after_message(&self) {
        self.update_build_diagnostics().await;
        for action in self.action_manager.take() {
            match action {
                Action::LoadDistribution => {
                    info!("Detected TeX distribution: {:?}", self.distribution.kind());
                    if self.distribution.kind() == DistributionKind::Unknown {
                        let params = ShowMessageParams {
                            message: "Your TeX distribution could not be detected. \
                                      Please make sure that your distribution is in your PATH."
                                .into(),
                            typ: MessageType::Error,
                        };
                        self.client.show_message(params).await;
                    }

                    if let Err(why) = self.distribution.load().await {
                        let message = match why {
                            texlab_distro::LoadError::KpsewhichNotFound => {
                                "An error occurred while executing `kpsewhich`.\
                                 Please make sure that your distribution is in your PATH \
                                 environment variable and provides the `kpsewhich` tool."
                            }
                            texlab_distro::LoadError::CorruptFileDatabase => {
                                "The file database of your TeX distribution seems \
                                 to be corrupt. Please rebuild it and try again."
                            }
                        };
                        let params = ShowMessageParams {
                            message: message.into(),
                            typ: MessageType::Error,
                        };
                        self.client.show_message(params).await;
                    };
                }
                Action::DetectRoot(uri) => {
                    if uri.scheme() == "file" {
                        let mut path = uri.to_file_path().unwrap();
                        while path.pop() {
                            let workspace = self.workspace_manager.get();
                            if workspace.find_parent(&uri).is_some() {
                                break;
                            }

                            for entry in WalkDir::new(&path)
                                .min_depth(1)
                                .max_depth(1)
                                .into_iter()
                                .filter_map(std::result::Result::ok)
                                .filter(|entry| entry.file_type().is_file())
                                .filter(|entry| {
                                    entry
                                        .path()
                                        .extension()
                                        .and_then(OsStr::to_str)
                                        .and_then(Language::by_extension)
                                        .is_some()
                                })
                            {
                                if let Ok(parent_uri) = Uri::from_file_path(entry.path()) {
                                    if workspace.find(&parent_uri).is_none() {
                                        let _ = self.workspace_manager.load(entry.path());
                                    }
                                }
                            }
                        }
                    }
                }
                Action::PublishDiagnostics => {
                    let workspace = self.workspace_manager.get();
                    for document in &workspace.documents {
                        let diagnostics = {
                            let manager = self.diagnostics_manager.lock().await;
                            manager.get(&document)
                        };

                        let params = PublishDiagnosticsParams {
                            uri: document.uri.clone().into(),
                            diagnostics,
                        };
                        self.client.publish_diagnostics(params).await;
                    }
                }
                Action::RunLinter(uri, reason) => {
                    let config: LatexLintOptions = self.configuration("latex.lint").await;
                    let should_lint = match reason {
                        LintReason::Change => config.on_change(),
                        LintReason::Save => config.on_save(),
                    };
                    if should_lint {
                        let workspace = self.workspace_manager.get();
                        if let Some(document) = workspace.find(&uri) {
                            if let SyntaxTree::Latex(_) = &document.tree {
                                let mut diagnostics_manager = self.diagnostics_manager.lock().await;
                                diagnostics_manager.latex.update(&uri, &document.text);
                            }
                        }
                    }
                }
                Action::Build(uri) => {
                    let config: LatexBuildOptions = self.configuration("latex.build").await;
                    if config.on_save() {
                        let text_document = TextDocumentIdentifier::new(uri.into());
                        self.build(BuildParams { text_document }).await.unwrap();
                    }
                }
                Action::CancelBuild(token) => {
                    self.build_manager.cancel(token).await;
                }
            }
        }
    }
}
