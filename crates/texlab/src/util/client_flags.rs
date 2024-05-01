/// Contains information about the client's capabilities.
/// This is used to determine which features the server can use.
#[derive(Debug, Clone)]
pub struct ClientFlags {
    /// If `true`, the server can return `DocumentSymbol` instead of `SymbolInformation`.
    pub hierarchical_document_symbols: bool,

    /// If `true`, the server can include markdown in completion items.
    /// This is used to include images via base64 encoding.
    pub completion_markdown: bool,

    /// If `true`, the server can include snippets like `\begin{...}` in completion items.
    pub completion_snippets: bool,

    /// The completion kinds supported by the client. Unsupported kinds will be replaced by `TEXT`.
    pub completion_kinds: Vec<lsp_types::CompletionItemKind>,

    /// If `true`, the server will always mark the completion list as incomplete.
    /// This is used as a workaround for VSCode where the client-side filtering messes with `filterText`.
    /// If not set, then fuzzy citation completion will not work.
    pub completion_always_incomplete: bool,

    /// If `true`, the server can include markdown in hover responses.
    pub hover_markdown: bool,

    /// If `true`, the server can pull the configuration from the client.
    pub configuration_pull: bool,

    /// If `true`, the client notifies the server when the configuration changes.
    pub configuration_push: bool,

    /// If `true`, the client can return `LocationLink` instead of `Location`.
    pub definition_link: bool,

    /// If `true`, the server can return custom kinds like `section`.
    pub folding_custom_kinds: bool,

    /// If `true`, the server can report progress using `WorkDoneProgress`.
    pub progress: bool,

    /// If `true`, the server can let the client open a document using `window/showDocument`.
    pub show_document: bool,
}
