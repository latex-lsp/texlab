use crate::completion::latex::data::types::LatexComponentDatabase;
use crate::completion::CompletionProvider;
use crate::definition::DefinitionProvider;
use crate::feature::FeatureRequest;
use crate::folding::FoldingProvider;
use crate::highlight::HighlightProvider;
use crate::hover::HoverProvider;
use crate::link::LinkProvider;
use crate::reference::ReferenceProvider;
use crate::rename::RenameProvider;
use crate::request;
use crate::workspace::WorkspaceManager;
use jsonrpc::Result;
use jsonrpc_derive::{jsonrpc_method, jsonrpc_server};
use log::*;
use lsp_types::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use walkdir::WalkDir;

pub struct LatexLspServer {
    workspace_manager: WorkspaceManager,
}

#[jsonrpc_server]
impl LatexLspServer {
    pub fn new() -> Self {
        LatexLspServer {
            workspace_manager: WorkspaceManager::new(),
        }
    }

    #[jsonrpc_method("initialize")]
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
                trigger_characters: Some(vec![
                    "\\".to_owned(),
                    "{".to_owned(),
                    "}".to_owned(),
                    "@".to_owned(),
                    "/".to_owned(),
                ]),
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
            document_formatting_provider: None,
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

    #[jsonrpc_method("initialized")]
    pub fn initialized(&self, params: InitializedParams) {}

    #[jsonrpc_method("shutdown")]
    pub async fn shutdown(&self, params: ()) -> Result<()> {
        Ok(())
    }

    #[jsonrpc_method("exit")]
    pub fn exit(&self, params: ()) {}

    #[jsonrpc_method("workspace/didChangeWatchedFiles")]
    pub fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {}

    #[jsonrpc_method("textDocument/didOpen")]
    pub fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.workspace_manager.add(params.text_document);
    }

    #[jsonrpc_method("textDocument/didChange")]
    pub fn did_change(&self, params: DidChangeTextDocumentParams) {
        for change in params.content_changes {
            let uri = params.text_document.uri.clone();
            self.workspace_manager.update(uri, change.text);
        }
    }

    #[jsonrpc_method("textDocument/didSave")]
    pub fn did_save(&self, params: DidSaveTextDocumentParams) {}

    #[jsonrpc_method("textDocument/didClose")]
    pub fn did_close(&self, params: DidCloseTextDocumentParams) {}

    #[jsonrpc_method("textDocument/completion")]
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

    #[jsonrpc_method("completionItem/resolve")]
    pub async fn completion_resolve(&self, item: CompletionItem) -> Result<CompletionItem> {
        Ok(item)
    }

    #[jsonrpc_method("textDocument/hover")]
    pub async fn hover(&self, params: TextDocumentPositionParams) -> Result<Option<Hover>> {
        let request = request!(self, params)?;
        let hover = await!(HoverProvider::execute(&request));
        Ok(hover)
    }

    #[jsonrpc_method("textDocument/definition")]
    pub async fn definition(&self, params: TextDocumentPositionParams) -> Result<Vec<Location>> {
        let request = request!(self, params)?;
        let results = await!(DefinitionProvider::execute(&request));
        Ok(results)
    }

    #[jsonrpc_method("textDocument/references")]
    pub async fn references(&self, params: ReferenceParams) -> Result<Vec<Location>> {
        let request = request!(self, params)?;
        let results = await!(ReferenceProvider::execute(&request));
        Ok(results)
    }

    #[jsonrpc_method("textDocument/documentHighlight")]
    pub async fn document_highlight(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Vec<DocumentHighlight>> {
        let request = request!(self, params)?;
        let results = await!(HighlightProvider::execute(&request));
        Ok(results)
    }

    #[jsonrpc_method("textDocument/documentSymbol")]
    pub async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Vec<DocumentSymbol>> {
        Ok(Vec::new())
    }

    #[jsonrpc_method("textDocument/documentLink")]
    pub async fn document_link(&self, params: DocumentLinkParams) -> Result<Vec<DocumentLink>> {
        let request = request!(self, params)?;
        let links = await!(LinkProvider::execute(&request));
        Ok(links)
    }

    #[jsonrpc_method("textDocument/formatting")]
    pub async fn formatting(&self, params: DocumentFormattingParams) -> Result<Vec<TextEdit>> {
        Ok(Vec::new())
    }

    #[jsonrpc_method("textDocument/rename")]
    pub async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let request = request!(self, params)?;
        let edit = await!(RenameProvider::execute(&request));
        Ok(edit)
    }

    #[jsonrpc_method("textDocument/foldingRange")]
    pub async fn folding_range(&self, params: FoldingRangeParams) -> Result<Vec<FoldingRange>> {
        let request = request!(self, params)?;
        let foldings = await!(FoldingProvider::execute(&request));
        Ok(foldings)
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

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    /// Defines how text documents are synced.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_document_sync: Option<TextDocumentSyncCapability>,

    /// The server provides hover support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hover_provider: Option<bool>,

    /// The server provides completion support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_provider: Option<CompletionOptions>,

    /// The server provides signature help support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_help_provider: Option<SignatureHelpOptions>,

    /// The server provides goto definition support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition_provider: Option<bool>,

    /// The server provides goto type definition support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_definition_provider: Option<TypeDefinitionProviderCapability>,

    /// the server provides goto implementation support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation_provider: Option<ImplementationProviderCapability>,

    /// The server provides find references support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references_provider: Option<bool>,

    /// The server provides document highlight support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_highlight_provider: Option<bool>,

    /// The server provides document symbol support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_symbol_provider: Option<bool>,

    /// The server provides workspace symbol support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_symbol_provider: Option<bool>,

    /// The server provides code actions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_action_provider: Option<CodeActionProviderCapability>,

    /// The server provides code lens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_lens_provider: Option<CodeLensOptions>,

    /// The server provides document formatting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_formatting_provider: Option<bool>,

    /// The server provides document range formatting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_range_formatting_provider: Option<bool>,

    /// The server provides document formatting on typing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_on_type_formatting_provider: Option<DocumentOnTypeFormattingOptions>,

    /// The server provides rename support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rename_provider: Option<RenameProviderCapability>,

    /// The server provides document link support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_link_provider: Option<DocumentLinkOptions>,

    /// The server provides color provider support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_provider: Option<ColorProviderCapability>,

    /// The server provides folding provider support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folding_range_provider: Option<FoldingRangeProviderCapability>,

    /// The server provides execute command support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execute_command_provider: Option<ExecuteCommandOptions>,

    /// Workspace specific server capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<WorkspaceCapability>,
}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
pub struct InitializeResult {
    /// The capabilities the language server provides.
    pub capabilities: ServerCapabilities,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentLinkOptions {
    /// Document links have a resolve provider as well.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolve_provider: Option<bool>,
}
