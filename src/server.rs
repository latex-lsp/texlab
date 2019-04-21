use lsp_types::*;

type LspResult<T> = Result<T, &'static str>;

pub struct LatexLspServer;

impl LatexLspServer {
    pub async fn initialize(&self, params: InitializeParams) -> LspResult<InitializeResult> {
        Ok(InitializeResult::default())
    }

    pub fn initialized(&self, params: InitializedParams) {}

    pub async fn shutdown(&self, params: ()) -> LspResult<()> {
        Ok(())
    }

    pub fn exit(&self, params: ()) {}

    pub fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {}

    pub fn did_open(&self, params: DidOpenTextDocumentParams) {}

    pub fn did_change(&self, params: DidChangeTextDocumentParams) {}

    pub fn did_save(&self, params: DidSaveTextDocumentParams) {}

    pub fn did_close(&self, params: DidCloseTextDocumentParams) {}

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
        Ok(Vec::new())
    }

    pub async fn references(&self, params: ReferenceParams) -> LspResult<Vec<Location>> {
        Ok(Vec::new())
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
        Ok(Vec::new())
    }
}
