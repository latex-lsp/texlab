use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use crossbeam_channel::Sender;
use log::{error, info, warn};
use lsp_server::{Connection, ErrorCode, Message, RequestId};
use lsp_types::{notification::*, request::*, *};
use serde::Serialize;
use threadpool::ThreadPool;

use crate::{
    client::{send_notification, send_request},
    component_db::COMPONENT_DATABASE,
    create_workspace_full,
    diagnostics::{DiagnosticsDebouncer, DiagnosticsManager, DiagnosticsMessage},
    dispatch::{NotificationDispatcher, RequestDispatcher},
    distro::Distribution,
    features::{
        find_all_references, find_document_highlights, find_document_links, find_document_symbols,
        find_foldings, find_hover, find_workspace_symbols, format_source_code, goto_definition,
        prepare_rename_all, rename_all, BuildEngine, BuildParams, BuildResult, BuildStatus,
        FeatureRequest, ForwardSearchResult, ForwardSearchStatus,
    },
    req_queue::{IncomingData, ReqQueue},
    ClientCapabilitiesExt, Document, DocumentLanguage, LineIndexExt, Options, ServerContext, Uri,
    Workspace, WorkspaceSource,
};

#[derive(Clone)]
pub struct Server {
    connection: Arc<Connection>,
    context: Arc<ServerContext>,
    req_queue: Arc<Mutex<ReqQueue>>,
    workspace: Arc<dyn Workspace>,
    static_debouncer: Arc<DiagnosticsDebouncer>,
    chktex_debouncer: Arc<DiagnosticsDebouncer>,
    pool: Arc<Mutex<ThreadPool>>,
    load_resolver: bool,
    build_engine: Arc<BuildEngine>,
}

impl Server {
    pub fn with_connection(
        connection: Connection,
        current_dir: PathBuf,
        load_resolver: bool,
    ) -> Result<Self> {
        let context = Arc::new(ServerContext::new(current_dir));
        let req_queue = Arc::default();
        let workspace = Arc::new(create_workspace_full(Arc::clone(&context))?);
        let diag_manager = Arc::new(Mutex::new(DiagnosticsManager::default()));

        let static_debouncer = Arc::new(create_static_debouncer(
            Arc::clone(&diag_manager),
            &connection,
            Arc::clone(&context),
        ));

        let chktex_debouncer = Arc::new(create_chktex_debouncer(
            diag_manager,
            &connection,
            Arc::clone(&context),
        ));

        Ok(Self {
            connection: Arc::new(connection),
            context,
            req_queue,
            workspace,
            static_debouncer,
            chktex_debouncer,
            pool: Arc::new(Mutex::new(threadpool::Builder::new().build())),
            load_resolver,
            build_engine: Arc::default(),
        })
    }

    fn spawn(&self, job: impl FnOnce() + Send + 'static) {
        self.pool.lock().unwrap().execute(job);
    }

    fn capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Options(
                TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(TextDocumentSyncKind::INCREMENTAL),
                    will_save: None,
                    will_save_wait_until: None,
                    save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                        include_text: Some(false),
                    })),
                },
            )),
            document_link_provider: Some(DocumentLinkOptions {
                resolve_provider: Some(false),
                work_done_progress_options: WorkDoneProgressOptions::default(),
            }),
            folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
            definition_provider: Some(OneOf::Left(true)),
            references_provider: Some(OneOf::Left(true)),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            #[cfg(feature = "completion")]
            completion_provider: Some(CompletionOptions {
                resolve_provider: Some(true),
                trigger_characters: Some(vec![
                    "\\".into(),
                    "{".into(),
                    "}".into(),
                    "@".into(),
                    "/".into(),
                    " ".into(),
                ]),
                ..CompletionOptions::default()
            }),
            document_symbol_provider: Some(OneOf::Left(true)),
            workspace_symbol_provider: Some(OneOf::Left(true)),
            rename_provider: Some(OneOf::Right(RenameOptions {
                prepare_provider: Some(true),
                work_done_progress_options: WorkDoneProgressOptions::default(),
            })),
            document_highlight_provider: Some(OneOf::Left(true)),
            document_formatting_provider: Some(OneOf::Left(true)),
            ..ServerCapabilities::default()
        }
    }

    fn initialize(&mut self) -> Result<()> {
        let (id, params) = self.connection.initialize_start()?;
        let params: InitializeParams = serde_json::from_value(params)?;

        *self.context.client_capabilities.lock().unwrap() = params.capabilities;
        *self.context.client_info.lock().unwrap() = params.client_info;

        let result = InitializeResult {
            capabilities: self.capabilities(),
            server_info: Some(ServerInfo {
                name: "TexLab".to_owned(),
                version: Some(env!("CARGO_PKG_VERSION").to_owned()),
            }),
        };
        self.connection
            .initialize_finish(id, serde_json::to_value(result)?)?;

        let cx = Arc::clone(&self.context);
        if self.load_resolver {
            self.spawn(move || {
                let distro = Distribution::detect();
                info!("Detected distribution: {}", distro.kind);
                *cx.resolver.lock().unwrap() = distro.resolver;
            });
        }

        self.register_diagnostics_handler();

        let server = self.clone();
        self.spawn(move || {
            server.register_config_capability();
            server.register_file_watching();
            server.pull_and_reparse_all();
        });

        Ok(())
    }

    fn register_file_watching(&self) {
        if self
            .context
            .client_capabilities
            .lock()
            .unwrap()
            .has_file_watching_support()
        {
            let options = DidChangeWatchedFilesRegistrationOptions {
                watchers: vec![FileSystemWatcher {
                    glob_pattern: "**/*.{aux,log}".into(),
                    kind: Some(WatchKind::Create | WatchKind::Change | WatchKind::Delete),
                }],
            };

            let reg = Registration {
                id: "build-watch".to_string(),
                method: DidChangeWatchedFiles::METHOD.to_string(),
                register_options: Some(serde_json::to_value(options).unwrap()),
            };

            let params = RegistrationParams {
                registrations: vec![reg],
            };

            if let Err(why) =
                send_request::<RegisterCapability>(&self.req_queue, &self.connection.sender, params)
            {
                error!(
                    "Failed to register \"{}\" notification: {}",
                    DidChangeWatchedFiles::METHOD,
                    why
                );
            }
        }
    }

    fn register_config_capability(&self) {
        let client_capabilities = self.context.client_capabilities.lock().unwrap();
        if client_capabilities.has_push_configuration_support() {
            drop(client_capabilities);
            let reg = Registration {
                id: "pull-config".to_string(),
                method: DidChangeConfiguration::METHOD.to_string(),
                register_options: None,
            };

            let params = RegistrationParams {
                registrations: vec![reg],
            };

            if let Err(why) =
                send_request::<RegisterCapability>(&self.req_queue, &self.connection.sender, params)
            {
                error!(
                    "Failed to register \"{}\" notification: {}",
                    DidChangeConfiguration::METHOD,
                    why
                );
            }
        }
    }

    fn register_diagnostics_handler(&mut self) {
        let sender = self.static_debouncer.sender.clone();
        self.workspace
            .register_open_handler(Arc::new(move |workspace, document| {
                let message = DiagnosticsMessage::Analyze {
                    workspace,
                    document,
                };
                sender.send(message).unwrap();
            }));
    }

    fn register_incoming_request(&self, id: RequestId) {
        let mut req_queue = self.req_queue.lock().unwrap();
        req_queue.incoming.register(id.clone(), IncomingData);
    }

    fn pull_config(&self) {
        if !self
            .context
            .client_capabilities
            .lock()
            .unwrap()
            .has_pull_configuration_support()
        {
            return;
        }

        let params = ConfigurationParams {
            items: vec![ConfigurationItem {
                section: Some("texlab".to_string()),
                scope_uri: None,
            }],
        };

        match send_request::<WorkspaceConfiguration>(
            &self.req_queue,
            &self.connection.sender,
            params,
        ) {
            Ok(mut json) => {
                let value = json.pop().expect("invalid configuration request");
                let new_options = match serde_json::from_value(value) {
                    Ok(new_options) => new_options,
                    Err(why) => {
                        warn!("Invalid configuration section \"texlab\": {}", why);
                        Options::default()
                    }
                };

                let mut options = self.context.options.write().unwrap();
                *options = new_options;
            }
            Err(why) => {
                error!("Retrieving configuration failed: {}", why);
            }
        };
    }

    fn cancel(&self, params: CancelParams) -> Result<()> {
        let id = match params.id {
            NumberOrString::Number(id) => RequestId::from(id),
            NumberOrString::String(id) => RequestId::from(id),
        };

        let mut req_queue = self.req_queue.lock().unwrap();
        req_queue.incoming.complete(id.clone());

        Ok(())
    }

    fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) -> Result<()> {
        for change in params.changes {
            if let Ok(path) = change.uri.to_file_path() {
                let uri = change.uri.into();
                match change.typ {
                    FileChangeType::CREATED | FileChangeType::CHANGED => {
                        self.workspace.reload(path)?;
                    }
                    FileChangeType::DELETED => {
                        self.workspace.delete(&uri);
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn did_change_configuration(&self, params: DidChangeConfigurationParams) -> Result<()> {
        let client_capabilities = { self.context.client_capabilities.lock().unwrap().clone() };
        if client_capabilities.has_pull_configuration_support() {
            let server = self.clone();
            self.spawn(move || {
                server.pull_and_reparse_all();
            });
        } else {
            match serde_json::from_value(params.settings) {
                Ok(new_options) => {
                    *self.context.options.write().unwrap() = new_options;
                }
                Err(why) => {
                    error!("Invalid configuration: {}", why);
                }
            };

            let server = self.clone();
            self.spawn(move || {
                server.reparse_all();
            });
        }

        Ok(())
    }

    fn did_open(&self, params: DidOpenTextDocumentParams) -> Result<()> {
        let language_id = &params.text_document.language_id;
        let language = DocumentLanguage::by_language_id(language_id);
        let document = self.workspace.open(
            Arc::new(params.text_document.uri.into()),
            params.text_document.text,
            language.unwrap_or(DocumentLanguage::Latex),
            WorkspaceSource::Client,
        );

        let should_lint = { self.context.options.read().unwrap().chktex.on_open_and_save };
        if let Some(document) = self
            .workspace
            .get(document.uri.as_ref())
            .filter(|_| should_lint)
        {
            self.chktex_debouncer
                .sender
                .send(DiagnosticsMessage::Analyze {
                    workspace: Arc::clone(&self.workspace),
                    document,
                })?;
        };
        Ok(())
    }

    fn did_change(&self, mut params: DidChangeTextDocumentParams) -> Result<()> {
        let uri = params.text_document.uri.into();
        let old_document = self.workspace.get(&uri);
        let old_text = old_document.as_ref().map(|document| document.text.as_str());
        let uri = Arc::new(uri);

        let language = self
            .workspace
            .get(&uri)
            .map(|document| document.data.language())
            .unwrap_or(DocumentLanguage::Latex);

        let new_document = match &old_document {
            Some(old_document) => params
                .content_changes
                .into_iter()
                .fold(Arc::clone(&old_document), |old_document, change| {
                    self.merge_text_changes(&old_document, language, change)
                }),
            None => self.workspace.open(
                Arc::clone(&uri),
                params.content_changes.pop().unwrap().text,
                language,
                WorkspaceSource::Client,
            ),
        };

        let line = match old_text {
            Some(old_text) => old_text
                .lines()
                .zip(new_document.text.lines())
                .position(|(a, b)| a != b)
                .unwrap_or_default() as u32,
            None => 0,
        };
        self.build_engine
            .positions_by_uri
            .insert(Arc::clone(&uri), Position::new(line, 0));

        let should_lint = { self.context.options.read().unwrap().chktex.on_edit };
        if should_lint {
            self.chktex_debouncer
                .sender
                .send(DiagnosticsMessage::Analyze {
                    workspace: Arc::clone(&self.workspace),
                    document: new_document,
                })?;
        };

        Ok(())
    }

    fn merge_text_changes(
        &self,
        old_document: &Document,
        new_language: DocumentLanguage,
        change: TextDocumentContentChangeEvent,
    ) -> Arc<Document> {
        let new_text = match change.range {
            Some(range) => {
                let range = old_document.line_index.offset_lsp_range(range);
                let mut new_text = String::new();
                new_text.push_str(&old_document.text[..range.start().into()]);
                new_text.push_str(&change.text);
                new_text.push_str(&old_document.text[range.end().into()..]);
                new_text
            }
            None => change.text,
        };

        self.workspace.open(
            Arc::clone(&old_document.uri),
            new_text,
            new_language,
            WorkspaceSource::Client,
        )
    }

    fn did_save(&self, params: DidSaveTextDocumentParams) -> Result<()> {
        let uri = params.text_document.uri.into();

        let should_build = { self.context.options.read().unwrap().build.on_save };
        if let Some(request) =
            self.workspace
                .get(&uri)
                .filter(|_| should_build)
                .and_then(|document| {
                    self.feature_request(
                        Arc::clone(&document.uri),
                        BuildParams {
                            text_document: TextDocumentIdentifier::new(uri.clone().into()),
                        },
                    )
                })
        {
            let lsp_sender = self.connection.sender.clone();
            let req_queue = Arc::clone(&self.req_queue);
            let build_engine = Arc::clone(&self.build_engine);
            self.spawn(move || {
                build_engine
                    .build(request, &req_queue, &lsp_sender)
                    .unwrap_or_else(|why| {
                        error!("Build failed: {}", why);
                        BuildResult {
                            status: BuildStatus::FAILURE,
                        }
                    });
            });
        }

        let should_lint = { self.context.options.read().unwrap().chktex.on_open_and_save };
        if let Some(document) = self.workspace.get(&uri).filter(|_| should_lint) {
            self.chktex_debouncer
                .sender
                .send(DiagnosticsMessage::Analyze {
                    workspace: Arc::clone(&self.workspace),
                    document,
                })?;
        };
        Ok(())
    }

    fn did_close(&self, params: DidCloseTextDocumentParams) -> Result<()> {
        let uri = params.text_document.uri.into();
        self.workspace.close(&uri);
        Ok(())
    }

    fn feature_request<P>(&self, uri: Arc<Uri>, params: P) -> Option<FeatureRequest<P>> {
        Some(FeatureRequest {
            context: Arc::clone(&self.context),
            params,
            workspace: Arc::clone(&self.workspace),
            subset: self.workspace.subset(uri)?,
        })
    }

    fn send_feature_error(&self, id: RequestId) -> Result<()> {
        let resp = lsp_server::Response::new_err(
            id,
            ErrorCode::InternalError as i32,
            "unknown document URI".to_string(),
        );
        self.connection.sender.send(resp.into())?;
        Ok(())
    }

    fn handle_feature_request<P, R, H>(
        &self,
        id: RequestId,
        params: P,
        uri: Arc<Uri>,
        handler: H,
    ) -> Result<()>
    where
        P: Send + 'static,
        R: Serialize,
        H: FnOnce(FeatureRequest<P>) -> R + Send + 'static,
    {
        match self.feature_request(uri, params) {
            Some(req) => {
                let sender = self.connection.sender.clone();
                self.spawn(move || {
                    let result = handler(req);
                    sender
                        .send(lsp_server::Response::new_ok(id, result).into())
                        .unwrap();
                });
            }
            None => {
                self.send_feature_error(id)?;
            }
        };
        Ok(())
    }

    fn document_link(&self, id: RequestId, params: DocumentLinkParams) -> Result<()> {
        let uri = Arc::new(params.text_document.uri.clone().into());
        self.handle_feature_request(id, params, uri, find_document_links)?;
        Ok(())
    }

    fn document_symbols(&self, id: RequestId, params: DocumentSymbolParams) -> Result<()> {
        let uri = Arc::new(params.text_document.uri.clone().into());
        self.handle_feature_request(id, params, uri, find_document_symbols)?;
        Ok(())
    }

    fn workspace_symbols(&self, id: RequestId, params: WorkspaceSymbolParams) -> Result<()> {
        let sender = self.connection.sender.clone();
        let workspace = Arc::clone(&self.workspace);
        self.spawn(move || {
            let result = find_workspace_symbols(workspace.as_ref(), &params);
            sender
                .send(lsp_server::Response::new_ok(id, result).into())
                .unwrap();
        });
        Ok(())
    }

    #[cfg(feature = "completion")]
    fn completion(&self, id: RequestId, params: CompletionParams) -> Result<()> {
        let uri = Arc::new(
            params
                .text_document_position
                .text_document
                .uri
                .clone()
                .into(),
        );

        self.build_engine
            .positions_by_uri
            .insert(Arc::clone(&uri), params.text_document_position.position);

        self.handle_feature_request(id, params, uri, crate::features::complete)?;
        Ok(())
    }

    #[cfg(feature = "completion")]
    fn completion_resolve(&self, id: RequestId, mut item: CompletionItem) -> Result<()> {
        let sender = self.connection.sender.clone();
        let workspace = Arc::clone(&self.workspace);
        self.spawn(move || {
            match serde_json::from_value(item.data.clone().unwrap()).unwrap() {
                crate::features::CompletionItemData::Package
                | crate::features::CompletionItemData::Class => {
                    item.documentation = COMPONENT_DATABASE
                        .documentation(&item.label)
                        .map(Documentation::MarkupContent);
                }
                #[cfg(feature = "citation")]
                crate::features::CompletionItemData::Citation { uri, key } => {
                    if let Some(document) = workspace.get(&uri) {
                        if let Some(data) = document.data.as_bibtex() {
                            let markup = crate::citation::render_citation(
                                &crate::syntax::bibtex::SyntaxNode::new_root(data.root.clone()),
                                &key,
                            );
                            item.documentation = markup.map(Documentation::MarkupContent);
                        }
                    }
                }
                _ => {}
            };

            drop(workspace);
            sender
                .send(lsp_server::Response::new_ok(id, item).into())
                .unwrap();
        });
        Ok(())
    }

    fn folding_range(&self, id: RequestId, params: FoldingRangeParams) -> Result<()> {
        let uri = Arc::new(params.text_document.uri.clone().into());
        self.handle_feature_request(id, params, uri, find_foldings)?;
        Ok(())
    }

    fn references(&self, id: RequestId, params: ReferenceParams) -> Result<()> {
        let uri = Arc::new(
            params
                .text_document_position
                .text_document
                .uri
                .clone()
                .into(),
        );
        self.handle_feature_request(id, params, uri, find_all_references)?;
        Ok(())
    }

    fn hover(&self, id: RequestId, params: HoverParams) -> Result<()> {
        let uri = Arc::new(
            params
                .text_document_position_params
                .text_document
                .uri
                .clone()
                .into(),
        );
        self.build_engine.positions_by_uri.insert(
            Arc::clone(&uri),
            params.text_document_position_params.position,
        );

        self.handle_feature_request(id, params, uri, find_hover)?;
        Ok(())
    }

    fn goto_definition(&self, id: RequestId, params: GotoDefinitionParams) -> Result<()> {
        let uri = Arc::new(
            params
                .text_document_position_params
                .text_document
                .uri
                .clone()
                .into(),
        );
        self.handle_feature_request(id, params, uri, goto_definition)?;
        Ok(())
    }

    fn prepare_rename(&self, id: RequestId, params: TextDocumentPositionParams) -> Result<()> {
        let uri = Arc::new(params.text_document.uri.clone().into());
        self.handle_feature_request(id, params, uri, prepare_rename_all)?;
        Ok(())
    }

    fn rename(&self, id: RequestId, params: RenameParams) -> Result<()> {
        let uri = Arc::new(
            params
                .text_document_position
                .text_document
                .uri
                .clone()
                .into(),
        );
        self.handle_feature_request(id, params, uri, rename_all)?;
        Ok(())
    }

    fn document_highlight(&self, id: RequestId, params: DocumentHighlightParams) -> Result<()> {
        let uri = Arc::new(
            params
                .text_document_position_params
                .text_document
                .uri
                .clone()
                .into(),
        );
        self.handle_feature_request(id, params, uri, find_document_highlights)?;
        Ok(())
    }

    fn formatting(&self, id: RequestId, params: DocumentFormattingParams) -> Result<()> {
        let uri = Arc::new(params.text_document.uri.clone().into());
        self.handle_feature_request(id, params, uri, format_source_code)?;
        Ok(())
    }

    fn semantic_tokens_range(
        &self,
        _id: RequestId,
        _params: SemanticTokensRangeParams,
    ) -> Result<()> {
        Ok(())
    }

    fn build(&self, id: RequestId, params: BuildParams) -> Result<()> {
        let uri = Arc::new(params.text_document.uri.clone().into());
        let lsp_sender = self.connection.sender.clone();
        let req_queue = Arc::clone(&self.req_queue);
        let build_engine = Arc::clone(&self.build_engine);
        self.handle_feature_request(id, params, uri, move |request| {
            build_engine
                .build(request, &req_queue, &lsp_sender)
                .unwrap_or_else(|why| {
                    error!("Build failed: {}", why);
                    BuildResult {
                        status: BuildStatus::FAILURE,
                    }
                })
        })?;
        Ok(())
    }

    fn forward_search(&self, id: RequestId, params: TextDocumentPositionParams) -> Result<()> {
        let uri = Arc::new(params.text_document.uri.clone().into());
        self.handle_feature_request(id, params, uri, |req| {
            crate::features::execute_forward_search(req).unwrap_or_else(|| ForwardSearchResult {
                status: ForwardSearchStatus::ERROR,
            })
        })?;
        Ok(())
    }

    fn pull_and_reparse_all(&self) {
        self.pull_config();
        self.reparse_all();
    }

    fn reparse_all(&self) {
        for document in self.workspace.documents() {
            self.workspace.open(
                Arc::clone(&document.uri),
                document.text.clone(),
                document.language(),
                WorkspaceSource::Client,
            );
        }
    }

    fn process_messages(&self) -> Result<()> {
        for msg in &self.connection.receiver {
            match msg {
                Message::Request(request) => {
                    if self.connection.handle_shutdown(&request)? {
                        return Ok(());
                    }

                    self.register_incoming_request(request.id.clone());
                    if let Some(response) = RequestDispatcher::new(request)
                        .on::<DocumentLinkRequest, _>(|id, params| self.document_link(id, params))?
                        .on::<FoldingRangeRequest, _>(|id, params| self.folding_range(id, params))?
                        .on::<References, _>(|id, params| self.references(id, params))?
                        .on::<HoverRequest, _>(|id, params| self.hover(id, params))?
                        .on::<DocumentSymbolRequest, _>(|id, params| {
                            self.document_symbols(id, params)
                        })?
                        .on::<WorkspaceSymbol, _>(|id, params| self.workspace_symbols(id, params))?
                        .on::<Completion, _>(|id, params| {
                            #[cfg(feature = "completion")]
                            self.completion(id, params)?;
                            Ok(())
                        })?
                        .on::<ResolveCompletionItem, _>(|id, params| {
                            #[cfg(feature = "completion")]
                            self.completion_resolve(id, params)?;
                            Ok(())
                        })?
                        .on::<GotoDefinition, _>(|id, params| self.goto_definition(id, params))?
                        .on::<PrepareRenameRequest, _>(|id, params| {
                            self.prepare_rename(id, params)
                        })?
                        .on::<Rename, _>(|id, params| self.rename(id, params))?
                        .on::<DocumentHighlightRequest, _>(|id, params| {
                            self.document_highlight(id, params)
                        })?
                        .on::<Formatting, _>(|id, params| self.formatting(id, params))?
                        .on::<BuildRequest, _>(|id, params| self.build(id, params))?
                        .on::<ForwardSearchRequest, _>(|id, params| {
                            self.forward_search(id, params)
                        })?
                        .on::<SemanticTokensRangeRequest, _>(|id, params| {
                            self.semantic_tokens_range(id, params)
                        })?
                        .default()
                    {
                        self.connection.sender.send(response.into())?;
                    }
                }
                Message::Notification(notification) => {
                    NotificationDispatcher::new(notification)
                        .on::<Cancel, _>(|params| self.cancel(params))?
                        .on::<DidChangeConfiguration, _>(|params| {
                            self.did_change_configuration(params)
                        })?
                        .on::<DidChangeWatchedFiles, _>(|params| {
                            self.did_change_watched_files(params)
                        })?
                        .on::<DidOpenTextDocument, _>(|params| self.did_open(params))?
                        .on::<DidChangeTextDocument, _>(|params| self.did_change(params))?
                        .on::<DidSaveTextDocument, _>(|params| self.did_save(params))?
                        .on::<DidCloseTextDocument, _>(|params| self.did_close(params))?
                        .default();
                }
                Message::Response(response) => {
                    let mut req_queue = self.req_queue.lock().unwrap();
                    let data = req_queue.outgoing.complete(response.id);
                    let result = match response.error {
                        Some(error) => Err(error),
                        None => Ok(response.result.unwrap_or_default()),
                    };
                    data.sender.send(result)?;
                }
            }
        }
        Ok(())
    }

    pub fn run(mut self) -> Result<()> {
        self.initialize()?;
        self.process_messages()?;
        drop(self.static_debouncer);
        drop(self.chktex_debouncer);
        self.pool.lock().unwrap().join();
        Ok(())
    }
}

fn create_static_debouncer(
    manager: Arc<Mutex<DiagnosticsManager>>,
    conn: &Connection,
    context: Arc<ServerContext>,
) -> DiagnosticsDebouncer {
    let sender = conn.sender.clone();
    DiagnosticsDebouncer::launch(context, move |workspace, document| {
        let mut manager = manager.lock().unwrap();
        manager.update_static(workspace.as_ref(), Arc::clone(&document.uri));
        if let Err(why) = publish_diagnostics(&sender, workspace.as_ref(), &manager) {
            warn!("Failed to publish diagnostics: {}", why);
        }
    })
}

fn create_chktex_debouncer(
    manager: Arc<Mutex<DiagnosticsManager>>,
    conn: &Connection,
    context: Arc<ServerContext>,
) -> DiagnosticsDebouncer {
    let sender = conn.sender.clone();
    DiagnosticsDebouncer::launch(Arc::clone(&context), move |workspace, document| {
        let options = { context.options.read().unwrap().clone() };
        let mut manager = manager.lock().unwrap();
        manager.update_chktex(workspace.as_ref(), Arc::clone(&document.uri), &options);
        if let Err(why) = publish_diagnostics(&sender, workspace.as_ref(), &manager) {
            warn!("Failed to publish diagnostics: {}", why);
        }
    })
}

fn publish_diagnostics(
    sender: &Sender<lsp_server::Message>,
    workspace: &dyn Workspace,
    diag_manager: &DiagnosticsManager,
) -> Result<()> {
    for document in workspace.documents() {
        let diagnostics = diag_manager.publish(Arc::clone(&document.uri));
        send_notification::<PublishDiagnostics>(
            sender,
            PublishDiagnosticsParams {
                uri: document.uri.as_ref().clone().into(),
                version: None,
                diagnostics,
            },
        )?;
    }
    Ok(())
}

struct BuildRequest;

impl lsp_types::request::Request for BuildRequest {
    type Params = BuildParams;

    type Result = BuildResult;

    const METHOD: &'static str = "textDocument/build";
}

struct ForwardSearchRequest;

impl lsp_types::request::Request for ForwardSearchRequest {
    type Params = TextDocumentPositionParams;

    type Result = ForwardSearchResult;

    const METHOD: &'static str = "textDocument/forwardSearch";
}
