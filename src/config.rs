use std::sync::{Mutex, RwLock};

use crossbeam_channel::Sender;
use log::{error, warn};
use lsp_server::Message;
use lsp_types::{
    notification::{DidChangeConfiguration, Notification},
    request::{RegisterCapability, WorkspaceConfiguration},
    ClientCapabilities, ConfigurationItem, ConfigurationParams, Registration, RegistrationParams,
};

use crate::{client::send_request, req_queue::ReqQueue, ClientCapabilitiesExt, Options};

pub fn register_config_capability(
    req_queue: &Mutex<ReqQueue>,
    sender: &Sender<Message>,
    client_capabilities: &Mutex<ClientCapabilities>,
) {
    let client_capabilities = client_capabilities.lock().unwrap();
    if !client_capabilities.has_pull_configuration_support()
        && client_capabilities.has_push_configuration_support()
    {
        drop(client_capabilities);
        let reg = Registration {
            id: "pull-config".to_string(),
            method: DidChangeConfiguration::METHOD.to_string(),
            register_options: None,
        };

        let params = RegistrationParams {
            registrations: vec![reg],
        };

        if let Err(why) = send_request::<RegisterCapability>(&req_queue, &sender, params) {
            error!(
                "Failed to register \"{}\" notification: {}",
                DidChangeConfiguration::METHOD,
                why
            );
        }
    }
}

pub fn pull_config(
    req_queue: &Mutex<ReqQueue>,
    sender: &Sender<Message>,
    options: &RwLock<Options>,
    client_capabilities: &ClientCapabilities,
) {
    if !client_capabilities.has_pull_configuration_support() {
        return;
    }

    let params = ConfigurationParams {
        items: vec![ConfigurationItem {
            section: Some("texlab".to_string()),
            scope_uri: None,
        }],
    };

    match send_request::<WorkspaceConfiguration>(req_queue, sender, params) {
        Ok(mut json) => {
            let value = json.pop().expect("invalid configuration request");
            let new_options = match serde_json::from_value(value) {
                Ok(new_options) => new_options,
                Err(why) => {
                    warn!("Invalid configuration section \"texlab\": {}", why);
                    Options::default()
                }
            };

            let mut options = options.write().unwrap();
            *options = new_options;
        }
        Err(why) => {
            error!("Retrieving configuration failed: {}", why);
        }
    };
}

pub fn push_config(options: &RwLock<Options>, config: serde_json::Value) {
    match serde_json::from_value(config) {
        Ok(new_options) => {
            *options.write().unwrap() = new_options;
        }
        Err(why) => {
            error!("Invalid configuration: {}", why);
        }
    };
}
