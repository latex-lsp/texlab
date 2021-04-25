use cancellation::CancellationTokenSource;
use crossbeam_channel::Sender;
use lsp_server::ResponseError;

pub struct IncomingData {
    pub(crate) token_source: CancellationTokenSource,
}

pub struct OutgoingData {
    pub(crate) sender: Sender<Result<serde_json::Value, ResponseError>>,
}

pub type ReqQueue = lsp_server::ReqQueue<IncomingData, OutgoingData>;
