use crate::client::LspClient;
use crate::completion::latex::data::types::LatexComponentDatabase;
use crate::completion::CompletionProvider;
use crate::definition::DefinitionProvider;
use crate::feature::FeatureRequest;
use crate::folding::FoldingProvider;
use crate::formatting::bibtex;
use crate::formatting::bibtex::{BibtexFormattingOptions, BibtexFormattingParams};
use crate::highlight::HighlightProvider;
use crate::hover::HoverProvider;
use crate::link::LinkProvider;
use crate::reference::ReferenceProvider;
use crate::rename::RenameProvider;
use crate::request;
use crate::syntax::bibtex::BibtexDeclaration;
use crate::syntax::text::SyntaxNode;
use crate::syntax::SyntaxTree;
use crate::workspace::WorkspaceManager;
use jsonrpc::server::Result;
use jsonrpc_derive::{jsonrpc_method, jsonrpc_server};
use log::*;
use lsp_types::*;
use serde::de::DeserializeOwned;
use std::borrow::Cow;
use std::sync::Arc;
use walkdir::WalkDir;

pub struct LatexLspServer<C> {
    client: Arc<C>,
    workspace_manager: WorkspaceManager,
}

#[jsonrpc_server]
impl<C: LspClient + Send + Sync> LatexLspServer<C> {
    pub fn new(client: Arc<C>) -> Self {
        LatexLspServer {
            client,
            workspace_manager: WorkspaceManager::new(),
        }
    }

    #[jsonrpc_method("initialize", kind = "request")]
    pub async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        if let Some(Ok(path)) = params.root_uri.map(|x| x.to_file_path()) {
            for entry in WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|x| x.file_type().is_file())
            {
                self.workspace_manager.load(&entry.path());
            }
        }

        let workspace = self.workspace_manager.get();
        workspace
            .documents
            .iter()
            .for_each(|x| info!("{}", x.uri.as_str()));

        let capabilities = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Options(
                TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(TextDocumentSyncKind::Full),
                    will_save: None,
                    will_save_wait_until: None,
                    save: None,
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
        };

        Ok(InitializeResult { capabilities })
    }

    #[jsonrpc_method("initialized", kind = "notification")]
    pub fn initialized(&self, params: InitializedParams) {}

    #[jsonrpc_method("shutdown", kind = "request")]
    pub async fn shutdown(&self, params: ()) -> Result<()> {
        Ok(())
    }

    #[jsonrpc_method("exit", kind = "notification")]
    pub fn exit(&self, params: ()) {}

    #[jsonrpc_method("workspace/didChangeWatchedFiles", kind = "notification")]
    pub fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {}

    #[jsonrpc_method("textDocument/didOpen", kind = "notification")]
    pub fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.workspace_manager.add(params.text_document);
    }

    #[jsonrpc_method("textDocument/didChange", kind = "notification")]
    pub fn did_change(&self, params: DidChangeTextDocumentParams) {
        for change in params.content_changes {
            let uri = params.text_document.uri.clone();
            self.workspace_manager.update(uri, change.text);
        }
    }

    #[jsonrpc_method("textDocument/didSave", kind = "notification")]
    pub fn did_save(&self, params: DidSaveTextDocumentParams) {}

    #[jsonrpc_method("textDocument/didClose", kind = "notification")]
    pub fn did_close(&self, params: DidCloseTextDocumentParams) {}

    #[jsonrpc_method("textDocument/completion", kind = "request")]
    pub async fn completion(&self, params: CompletionParams) -> Result<CompletionList> {
        let request = request!(self, params)?;
        let items = await!(CompletionProvider::execute(&request));
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
    pub async fn completion_resolve(&self, item: CompletionItem) -> Result<CompletionItem> {
        Ok(item)
    }

    #[jsonrpc_method("textDocument/hover", kind = "request")]
    pub async fn hover(&self, params: TextDocumentPositionParams) -> Result<Option<Hover>> {
        let request = request!(self, params)?;
        let hover = await!(HoverProvider::execute(&request));
        Ok(hover)
    }

    #[jsonrpc_method("textDocument/definition", kind = "request")]
    pub async fn definition(&self, params: TextDocumentPositionParams) -> Result<Vec<Location>> {
        let request = request!(self, params)?;
        let results = await!(DefinitionProvider::execute(&request));
        Ok(results)
    }

    #[jsonrpc_method("textDocument/references", kind = "request")]
    pub async fn references(&self, params: ReferenceParams) -> Result<Vec<Location>> {
        let request = request!(self, params)?;
        let results = await!(ReferenceProvider::execute(&request));
        Ok(results)
    }

    #[jsonrpc_method("textDocument/documentHighlight", kind = "request")]
    pub async fn document_highlight(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Vec<DocumentHighlight>> {
        let request = request!(self, params)?;
        let results = await!(HighlightProvider::execute(&request));
        Ok(results)
    }

    #[jsonrpc_method("textDocument/documentSymbol", kind = "request")]
    pub async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Vec<DocumentSymbol>> {
        Ok(Vec::new())
    }

    #[jsonrpc_method("textDocument/documentLink", kind = "request")]
    pub async fn document_link(&self, params: DocumentLinkParams) -> Result<Vec<DocumentLink>> {
        let request = request!(self, params)?;
        let links = await!(LinkProvider::execute(&request));
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
                options: await!(self.configuration::<BibtexFormattingOptions>("bibtex.formatting")),
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
        let edit = await!(RenameProvider::execute(&request));
        Ok(edit)
    }

    #[jsonrpc_method("textDocument/foldingRange", kind = "request")]
    pub async fn folding_range(&self, params: FoldingRangeParams) -> Result<Vec<FoldingRange>> {
        let request = request!(self, params)?;
        let foldings = await!(FoldingProvider::execute(&request));
        Ok(foldings)
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

        match await!(self.client.configuration(params)) {
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

#[macro_export]
macro_rules! request {
    ($server:expr, $params:expr) => {{
        let workspace = $server.workspace_manager.get();
        if let Some(document) = workspace.find(&$params.text_document.uri) {
            Ok(FeatureRequest::new(
                $params,
                workspace,
                document,
                Arc::new(LatexComponentDatabase::default()),
            ))
        } else {
            let msg = format!("Unknown document: {}", $params.text_document.uri);
            Err(msg)
        }
    }};
}
