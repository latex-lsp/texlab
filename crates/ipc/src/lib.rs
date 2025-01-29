use std::{
    io::{self, BufRead, BufReader, BufWriter, Read},
    path::PathBuf,
};

use serde::{de::DeserializeOwned, Serialize};

#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};

#[cfg(windows)]
use uds_windows::{UnixListener, UnixStream};

fn socket_path() -> PathBuf {
    dirs::runtime_dir()
        .unwrap_or(std::env::temp_dir())
        .join("texlab.sock")
}

pub fn send_request<T: Serialize>(msg: T) -> io::Result<()> {
    let stream = UnixStream::connect(socket_path())?;
    let mut conn = BufWriter::new(stream);
    serde_json::to_writer(&mut conn, &msg)?;
    Ok(())
}

pub fn spawn_server<T, F>(mut event_handler: F) -> io::Result<()>
where
    T: DeserializeOwned,
    F: FnMut(T) + Send + 'static,
{
    let socket_path = socket_path();
    let _ = std::fs::remove_file(&socket_path);
    let listener = UnixListener::bind(socket_path)?;

    std::thread::spawn(move || {
        for conn in listener.incoming().flatten() {
            let _ = handle_request(conn, &mut event_handler);
        }
    });

    Ok(())
}

fn handle_request<T, F>(conn: impl Read, event_handler: &mut F) -> io::Result<()>
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
