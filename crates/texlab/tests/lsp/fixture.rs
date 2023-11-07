use std::{
    path::{Path, PathBuf},
    sync::Once,
    thread::JoinHandle,
};

use anyhow::Result;
use distro::Language;
use lsp_server::Connection;
use lsp_types::{
    notification::{DidOpenTextDocument, Exit, Initialized},
    request::{Initialize, Shutdown},
    ClientCapabilities, DidOpenTextDocumentParams, InitializeParams, InitializedParams, Position,
    Range, TextDocumentIdentifier, TextDocumentItem, TextDocumentPositionParams, Url,
};
use tempfile::{tempdir, TempDir};
use texlab::{LspClient, Server};

#[derive(Debug)]
pub struct Fixture {
    pub documents: Vec<Document>,
}

impl Fixture {
    pub fn parse(input: &str) -> Fixture {
        let mut documents = Vec::new();

        let mut start = 0;
        if !input.is_empty() {
            for end in input
                .match_indices("%!")
                .skip(1)
                .map(|(i, _)| i)
                .chain(std::iter::once(input.len()))
            {
                documents.push(Document::parse(&input[start..end]));
                start = end;
            }
        }

        Self { documents }
    }

    pub fn setup(&self, client: &LspClient, dir: &Path) -> Result<()> {
        for document in &self.documents {
            let text = String::from(&document.text);
            let path = dir.join(&document.path);
            std::fs::create_dir_all(path.parent().unwrap())?;
            std::fs::write(&path, &text)?;

            let uri = Url::from_file_path(&path).unwrap();
            let language = Language::from_path(&path).unwrap_or(Language::Tex);
            let language_id = String::from(match language {
                Language::Tex => "latex",
                Language::Bib => "bibtex",
                Language::Aux | Language::Log | Language::Root | Language::Tectonic => continue,
            });

            client.send_notification::<DidOpenTextDocument>(DidOpenTextDocumentParams {
                text_document: TextDocumentItem::new(uri, language_id, 0, text),
            })?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Document {
    pub path: PathBuf,
    pub text: String,
    pub cursor: Option<Position>,
    pub ranges: Vec<Range>,
}

impl Document {
    pub fn parse(input: &str) -> Self {
        let mut lines = Vec::new();

        let (path, input) = input
            .trim()
            .strip_prefix("%! ")
            .map(|input| input.split_once('\n').unwrap_or((input, "")))
            .unwrap();

        let mut ranges = Vec::new();
        let mut cursor = None;

        for line in input.lines().map(|line| line.trim_end()) {
            if line.chars().all(|c| matches!(c, ' ' | '^' | '|' | '!')) && !line.is_empty() {
                let index = (lines.len() - 1) as u32;

                cursor = cursor.or_else(|| {
                    let character = line.find('|')?;
                    Some(Position::new(index, character as u32))
                });

                if let Some(start) = line.find('!') {
                    let position = Position::new(index, start as u32);
                    ranges.push(Range::new(position, position));
                }

                if let Some(start) = line.find('^') {
                    let end = line.rfind('^').unwrap() + 1;
                    ranges.push(Range::new(
                        Position::new(index, start as u32),
                        Position::new(index, end as u32),
                    ));
                }
            } else {
                lines.push(line);
            }
        }

        Self {
            path: PathBuf::from(path),
            text: lines.join("\n"),
            cursor,
            ranges,
        }
    }
}

static LOGGER: Once = Once::new();

#[derive(Debug)]
pub struct TestBed {
    fixture: Fixture,
    _temp_dir: TempDir,
    temp_dir_path: PathBuf,
    client: LspClient,
    client_thread: Option<JoinHandle<()>>,
    server_thread: Option<JoinHandle<()>>,
}

impl Drop for TestBed {
    fn drop(&mut self) {
        let _ = self.client.send_request::<Shutdown>(());
        let _ = self.client.send_notification::<Exit>(());
        self.client_thread.take().unwrap().join().unwrap();
        self.server_thread.take().unwrap().join().unwrap();
    }
}

impl TestBed {
    pub fn new(fixture: &str) -> Result<Self> {
        LOGGER.call_once(|| {
            if option_env!("TEST_LOG") == Some("1") {
                fern::Dispatch::new()
                    .filter(|metadata| {
                        metadata.target().contains("texlab")
                            || metadata.target().contains("lsp_server")
                    })
                    .level(log::LevelFilter::Trace)
                    .chain(std::io::stderr())
                    .apply()
                    .unwrap()
            }
        });

        let fixture = Fixture::parse(fixture);
        let (server_conn, client_conn) = Connection::memory();

        let client = LspClient::new(client_conn.sender);

        let server_thread = std::thread::spawn(move || Server::new(server_conn).run().unwrap());
        let client_thread = {
            let client = client.clone();
            std::thread::spawn(move || {
                for message in &client_conn.receiver {
                    match message {
                        lsp_server::Message::Request(request) => {
                            client
                                .send_error(
                                    request.id,
                                    lsp_server::ErrorCode::MethodNotFound.into(),
                                    "Method not found".into(),
                                )
                                .unwrap();
                        }
                        lsp_server::Message::Response(response) => {
                            client.recv_response(response).unwrap();
                        }
                        lsp_server::Message::Notification(_) => {}
                    }
                }
            })
        };

        let temp_dir = tempdir()?;
        let temp_dir_path = temp_dir.path().canonicalize()?;

        Ok(TestBed {
            fixture,
            _temp_dir: temp_dir,
            temp_dir_path,
            client,
            client_thread: Some(client_thread),
            server_thread: Some(server_thread),
        })
    }

    pub fn initialize(&self, capabilities: ClientCapabilities) -> Result<()> {
        self.client.send_request::<Initialize>(InitializeParams {
            capabilities,
            initialization_options: Some(serde_json::json!({ "skipDistro": true })),
            ..Default::default()
        })?;

        self.client
            .send_notification::<Initialized>(InitializedParams {})?;

        self.fixture.setup(&self.client, self.directory())?;
        Ok(())
    }

    pub fn client(&self) -> &LspClient {
        &self.client
    }

    pub fn cursor(&self) -> Option<TextDocumentPositionParams> {
        let (document, cursor) = self
            .fixture
            .documents
            .iter()
            .find_map(|document| document.cursor.map(|cursor| (document, cursor)))?;

        let uri = Url::from_file_path(self.temp_dir_path.join(&document.path)).unwrap();
        let id = TextDocumentIdentifier::new(uri);
        Some(TextDocumentPositionParams::new(id, cursor))
    }

    pub fn directory(&self) -> &Path {
        &self.temp_dir_path
    }

    pub fn redact(&self, uri: &Url) -> Url {
        if uri.scheme() != "file" {
            return uri.clone();
        }

        let path = uri.to_file_path().unwrap().canonicalize().unwrap();
        let path = path.strip_prefix(self.directory()).unwrap();
        let path = path.to_str().unwrap();

        let text = format!("redacted://{}", path.replace('\\', "/"));
        Url::parse(&text).unwrap()
    }
}
