use std::{
    io::{self, BufRead, BufReader, BufWriter},
    thread::JoinHandle,
};

use anyhow::Result;
use crossbeam_channel::Receiver;
use interprocess::local_socket::{prelude::*, GenericNamespaced, ListenerOptions, Stream};
use lsp_types::Url;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub uri: Url,
    pub line: u32,
}

const SOCKET_NAME: &str = "texlab.sock";

pub struct Client;

pub fn send_request(request: Request) -> Result<()> {
    let name = SOCKET_NAME.to_ns_name::<GenericNamespaced>().unwrap();
    let conn = Stream::connect(name)?;
    let mut conn = BufWriter::new(conn);
    let _ = serde_json::to_writer(&mut conn, &request);
    Ok(())
}

pub fn spawn_server() -> Result<(JoinHandle<()>, Receiver<Request>)> {
    let (tx, rx) = crossbeam_channel::unbounded();

    if cfg!(unix) {
        let sock_file = std::env::temp_dir().join(format!("{SOCKET_NAME}.sock"));
        let _ = std::fs::remove_file(sock_file);
    }

    let name = SOCKET_NAME.to_ns_name::<GenericNamespaced>()?;
    let opts = ListenerOptions::new().name(name);
    let listener = match opts.create_sync() {
        Err(e) if e.kind() == io::ErrorKind::AddrInUse => {
            log::warn!("Unable to open socket {SOCKET_NAME} because it is already in use.");
            return Err(e.into());
        }
        x => x?,
    };

    let join_handle = std::thread::spawn(move || {
        for conn in listener.incoming().flatten() {
            let _ = handle_request(conn, &tx);
        }
    });

    Ok((join_handle, rx))
}

fn handle_request(conn: Stream, tx: &crossbeam_channel::Sender<Request>) -> Result<()> {
    let mut conn = BufReader::new(conn);
    let mut line = String::new();
    conn.read_line(&mut line)?;
    let request = serde_json::from_str(&line)?;
    tx.send(request)?;
    Ok(())
}
