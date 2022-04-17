use crossbeam_channel::Sender;
use lsp_server::ResponseError;

pub struct IncomingData;

pub struct OutgoingData {
    pub(crate) sender: Sender<Result<serde_json::Value, ResponseError>>,
}

pub type ReqQueue = lsp_server::ReqQueue<IncomingData, OutgoingData>;
