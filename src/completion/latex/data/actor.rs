use crate::completion::latex::data::types::{LatexComponent, LatexComponentDatabase};
use crate::workspace::SyntaxTree;
use futures::channel::{mpsc, oneshot};
use futures::executor::ThreadPool;
use futures::lock::Mutex;
use futures::prelude::*;
use futures::task::*;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

enum Message {
    Get(oneshot::Sender<Arc<LatexComponentDatabase>>),
    Add(LatexComponent),
}

pub struct LatexComponentDatabaseActor {
    sender: Mutex<mpsc::Sender<Message>>,
    receiver: Mutex<mpsc::Receiver<Message>>,
}

impl LatexComponentDatabaseActor {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(0);
        LatexComponentDatabaseActor {
            sender: Mutex::new(sender),
            receiver: Mutex::new(receiver),
        }
    }

    pub async fn spawn(mut pool: ThreadPool, path: PathBuf) -> Arc<Self> {
        let actor = Arc::new(Self::new());
        let task = |actor: Arc<LatexComponentDatabaseActor>| {
            async move {
                let mut database = Arc::new(Self::load_database(&path).unwrap_or_default());
                let mut receiver = await!(actor.receiver.lock());
                while let Some(message) = await!(receiver.next()) {
                    match message {
                        Message::Get(sender) => {
                            let database = Arc::clone(&database);
                            sender.send(database).unwrap();
                        }
                        Message::Add(component) => {
                            let mut components = Vec::new();
                            for component in &database.components {
                                components.push(Arc::clone(&component));
                            }
                            components.push(Arc::new(component));
                            database = Arc::new(LatexComponentDatabase::new(components));
                            Self::save_database(&path, &database);
                        }
                    }
                }
            }
        };
        pool.spawn(task(Arc::clone(&actor)))
            .expect("Failed to intitialize completion database");
        actor
    }

    fn load_database(path: &Path) -> Option<LatexComponentDatabase> {
        let mut file = std::fs::File::open(path).ok()?;
        let mut text = String::new();
        file.read_to_string(&mut text).ok()?;
        serde_json::from_str(&text).ok()
    }

    fn save_database(path: &Path, database: &LatexComponentDatabase) {
        let mut file = std::fs::File::create(path).expect("Failed to create completion database");
        let text = serde_json::to_string_pretty(database)
            .expect("Failed to serialize completion database");
        file.write_all(&mut text.into_bytes())
            .expect("Failed to save completion database");
    }

    pub async fn get(&self) -> Arc<LatexComponentDatabase> {
        let (sender, receiver) = oneshot::channel();
        let message = Message::Get(sender);
        await!(self.send(message));
        await!(receiver).unwrap()
    }

    pub async fn add(&self, component: LatexComponent) {
        let message = Message::Add(component);
        await!(self.send(message));
    }

    async fn send(&self, message: Message) {
        let mut sender = await!(self.sender.lock());
        await!(sender.send(message)).unwrap();
    }
}
