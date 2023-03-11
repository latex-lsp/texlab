use crate::Config;

/// Contains the global context of the server throughout the application.
#[salsa::input(singleton)]
pub struct ServerContext {
    /// The server configuration which is extracted from either
    /// the `workspace/configuration` or `workspace/didChangeConfiguration` messages.
    #[return_ref]
    pub config: Config,

    /// Disable usage of `isIncomplete = false` in completion lists.
    ///
    /// Due to the large number of completion results,
    /// the server can only send a subset of the items most of the time.
    /// When the filtered list is small enough, `CompletionList.isIncomplete` can be set to `false`.
    /// On VSCode, this optimization should not be done so this flag is needed.
    pub always_incomplete_completion_list: bool,
}
