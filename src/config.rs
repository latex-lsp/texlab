use futures::lock::Mutex;
use futures_boxed::boxed;
use log::*;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use texlab_protocol::*;

pub trait ConfigStrategy: Send + Sync {
    #[boxed]
    async fn get(&self, fetch: bool) -> Options;

    #[boxed()]
    async fn set(&self, settings: serde_json::Value);
}

impl dyn ConfigStrategy {
    pub fn select<C: LspClient + Send + Sync + 'static>(
        capabilities: &ClientCapabilities,
        client: Arc<C>,
    ) -> Box<Self> {
        if capabilities.has_pull_configuration_support() {
            Box::new(PullConfigStrategy::new(client))
        } else if capabilities.has_push_configuration_support() {
            Box::new(PushConfigStrategy::new())
        } else {
            Box::new(NoConfigStrategy::new())
        }
    }
}

#[derive(Debug)]
struct PullConfigStrategy<C> {
    client: Arc<C>,
    options: Mutex<Options>,
}

impl<C: LspClient> PullConfigStrategy<C> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            options: Mutex::default(),
        }
    }

    async fn configuration<T>(&self, section: &'static str) -> T
    where
        T: DeserializeOwned + Default,
    {
        let params = ConfigurationParams {
            items: vec![ConfigurationItem {
                section: Some(section.into()),
                scope_uri: None,
            }],
        };

        match self.client.configuration(params).await {
            Ok(json) => match serde_json::from_value::<Vec<T>>(json) {
                Ok(config) => config.into_iter().next().unwrap(),
                Err(_) => {
                    warn!("Invalid configuration: {}", section);
                    T::default()
                }
            },
            Err(why) => {
                error!(
                    "Retrieving configuration for {} failed: {}",
                    section, why.message
                );
                T::default()
            }
        }
    }
}

impl<C: LspClient + Send + Sync> ConfigStrategy for PullConfigStrategy<C> {
    #[boxed]
    async fn get(&self, fetch: bool) -> Options {
        if fetch {
            let options = Options {
                latex: Some(self.configuration("latex").await),
                bibtex: Some(self.configuration("bibtex").await),
            };
            let mut options_guard = self.options.lock().await;
            *options_guard = options;
        }
        self.options.lock().await.clone()
    }

    #[boxed]
    async fn set(&self, _settings: serde_json::Value) {}
}

#[derive(Debug, Default)]
struct PushConfigStrategy {
    options: Mutex<Options>,
}

impl PushConfigStrategy {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ConfigStrategy for PushConfigStrategy {
    #[boxed]
    async fn get(&self, _fetch: bool) -> Options {
        let options = self.options.lock().await;
        options.clone()
    }

    #[boxed()]
    async fn set(&self, settings: serde_json::Value) {
        let mut options = self.options.lock().await;
        match serde_json::from_value(settings) {
            Ok(settings) => *options = settings,
            Err(why) => warn!("Invalid configuration: {}", why),
        }
    }
}

#[derive(Debug, Default)]
struct NoConfigStrategy;

impl NoConfigStrategy {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ConfigStrategy for NoConfigStrategy {
    #[boxed]
    async fn get(&self, _fetch: bool) -> Options {
        Options::default()
    }

    #[boxed()]
    async fn set(&self, _settings: serde_json::Value) {}
}
