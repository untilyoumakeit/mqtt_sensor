extern crate futures;
extern crate paho_mqtt as mqtt;
extern crate json;
use serde::{Serialize, Deserialize};

use std::time::Duration;
use std::collections::BTreeMap;

// Use a non-zero QOS to exercise the persistence store
const QOS: i32 = 1;

pub trait Message: Send {
    fn write(&self, map: &mut BTreeMap<String, String>);
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Configuration {
    host: String,
    username: Option<String>,
    password: Option<String>,
}

impl Configuration {
    fn name(&self) -> String {
        "raspberrypi".to_string() // TODO Get it from yaml
    }

    fn client_id(&self) -> String {
        "sysinfo2mqtt".to_string()
    }
}

pub struct Client {
    topic: String,
    cli: mqtt::AsyncClient,
}

impl Client {
    pub fn new(config: &Configuration) -> Client {
        let host = &config.host;
        let opts = mqtt::CreateOptionsBuilder::new()
            .client_id(config.name())
            .server_uri(host)
            .finalize();
        let cli = mqtt::AsyncClient::new(opts).unwrap_or_else(|_| {
            // TODO Log it and exit process
            panic!("Can't create a MQTT client");
        });
        let mut opts = mqtt::ConnectOptionsBuilder::new();
        if let Some(username) = &config.username {
            opts.user_name(username);
        }
        if let Some(password) = &config.password {
            opts.password(password);
        }
        opts.keep_alive_interval(Duration::from_secs(15));
        opts.clean_session(false);
        let opts = opts.finalize();
        // TODO check is connection can be set
        cli.connect(opts);
        // connect_token
        let topic = format!("{}/{}", config.client_id(), config.name());
        Client { topic, cli }
    }
    
    pub fn send<T: Message + Sized + 'static>(&self, messages: Vec<T>) {
        let mut data: BTreeMap<String, String> = BTreeMap::new();

        for m in messages {
            m.write(&mut data);
        }
        let data = json::from(data);

        let payload = json::object!{
            "action" => "update",
            "values" => data
        };

        let message = mqtt::MessageBuilder::new()
            .retained(true)
            .topic(&self.topic)
            .qos(QOS)
            .payload(payload.to_string())
            .finalize();
        self.cli.publish(message);
        println!("Send vector of messages");
    }
}
