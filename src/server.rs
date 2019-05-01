use crate::definition::DefinitionProvider;
use crate::feature::FeatureRequest;
use crate::folding::FoldingProvider;
use crate::reference::ReferenceProvider;
use crate::request;
use crate::workspace::WorkspaceActor;
use log::*;
use lsp_types::*;
use std::sync::Arc;
use walkdir::WalkDir;

type LspResult<T> = Result<T, String>;

pub struct LatexLspServer {
    workspace: Arc<WorkspaceActor>,
}

impl LatexLspServer {
    pub fn new(workspace: Arc<WorkspaceActor>) -> Self {
        LatexLspServer { workspace }
    }

    pub async fn initialize(&self, params: InitializeParams) -> LspResult<InitializeResult> {
        if let Some(Ok(path)) = params.root_uri.map(|x| x.to_file_path()) {
            for entry in WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|x| x.file_type().is_file())
            {
                await!(self.workspace.load(entry.path().to_path_buf()));
            }
        }

        let workspace = await!(self.workspace.get());
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
            hover_provider: None,
            completion_provider: None,
            signature_help_provider: None,
            definition_provider: Some(true),
            type_definition_provider: None,
            implementation_provider: None,
            references_provider: Some(true),
            document_highlight_provider: None,
            document_symbol_provider: None,
            workspace_symbol_provider: None,
            code_action_provider: None,
            code_lens_provider: None,
            document_formatting_provider: None,
            document_range_formatting_provider: None,
            document_on_type_formatting_provider: None,
            rename_provider: None,
            color_provider: None,
            folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
            execute_command_provider: None,
            workspace: None,
        };

        Ok(InitializeResult { capabilities })
    }

    pub async fn initialized(&self, params: InitializedParams) {}

    pub async fn shutdown(&self, params: ()) -> LspResult<()> {
        Ok(())
    }

    pub async fn exit(&self, params: ()) {}

    pub async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {}

    pub async fn did_open(&self, params: DidOpenTextDocumentParams) {
        await!(self.workspace.add(params.text_document));
    }

    pub async fn did_change(&self, params: DidChangeTextDocumentParams) {
        for change in params.content_changes {
            let uri = params.text_document.uri.clone();
            await!(self.workspace.update(uri, change.text))
        }
    }

    pub async fn did_save(&self, params: DidSaveTextDocumentParams) {}

    pub async fn did_close(&self, params: DidCloseTextDocumentParams) {}

    pub async fn completion(&self, params: CompletionParams) -> LspResult<CompletionList> {
        Ok(CompletionList::default())
    }

    pub async fn completion_resolve(&self, item: CompletionItem) -> LspResult<CompletionItem> {
        Ok(item)
    }

    pub async fn hover(&self, params: TextDocumentPositionParams) -> LspResult<Option<Hover>> {
        Ok(None)
    }

    pub async fn definition(&self, params: TextDocumentPositionParams) -> LspResult<Vec<Location>> {
        let request = request!(self, params)?;
        let results = await!(DefinitionProvider::execute(&request));
        Ok(results)
    }

    pub async fn references(&self, params: ReferenceParams) -> LspResult<Vec<Location>> {
        let request = request!(self, params)?;
        let results = await!(ReferenceProvider::execute(&request));
        Ok(results)
    }

    pub async fn document_highlight(
        &self,
        params: TextDocumentPositionParams,
    ) -> LspResult<Vec<DocumentHighlight>> {
        Ok(Vec::new())
    }

    pub async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> LspResult<Vec<DocumentSymbol>> {
        Ok(Vec::new())
    }

    pub async fn document_link(&self, params: DocumentLinkParams) -> LspResult<Vec<DocumentLink>> {
        Ok(Vec::new())
    }

    pub async fn formatting(&self, params: DocumentFormattingParams) -> LspResult<Vec<TextEdit>> {
        Ok(Vec::new())
    }

    pub async fn rename(&self, params: RenameParams) -> LspResult<Option<WorkspaceEdit>> {
        Ok(None)
    }

    pub async fn folding_range(&self, params: FoldingRangeParams) -> LspResult<Vec<FoldingRange>> {
        let request = request!(self, params)?;
        let foldings = await!(FoldingProvider::execute(&request));
        Ok(foldings)
    }
}

#[macro_export]
macro_rules! request {
    ($server:expr, $params:expr) => {{
        let workspace = await!($server.workspace.get());
        if let Some(document) = workspace.find(&$params.text_document.uri) {
            Ok(FeatureRequest::new($params, workspace, document))
        } else {
            let msg = format!("Unknown document: {}", $params.text_document.uri);
            Err(msg)
        }
    }};
}
