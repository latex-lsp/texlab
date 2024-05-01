use std::io::{self, BufRead, BufReader, BufWriter};

use interprocess::local_socket::{prelude::*, GenericNamespaced, ListenerOptions, Stream};
use serde::{de::DeserializeOwned, Serialize};

const SOCKET_NAME: &str = "texlab.sock";

pub fn send_request<T: Serialize>(msg: T) -> io::Result<()> {
    let name = SOCKET_NAME.to_ns_name::<GenericNamespaced>()?;
    let conn = Stream::connect(name)?;
    let mut conn = BufWriter::new(conn);
    serde_json::to_writer(&mut conn, &msg)?;
    Ok(())
}

pub fn spawn_server<T, F>(mut event_handler: F) -> io::Result<()>
where
    T: DeserializeOwned,
    F: FnMut(T) + Send + 'static,
{
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

    std::thread::spawn(move || {
        for conn in listener.incoming().flatten() {
            let _ = handle_request(conn, &mut event_handler);
        }
    });

    Ok(())
}

fn handle_request<T, F>(conn: Stream, event_handler: &mut F) -> io::Result<()>
where
    T: DeserializeOwned,
    F: FnMut(T),
{
    let mut conn = BufReader::new(conn);
    let mut line = String::new();
    conn.read_line(&mut line)?;
    let msg = serde_json::from_str(&line)?;
    event_handler(msg);
    Ok(())
}
