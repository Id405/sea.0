use std::path::Path;

use irc::client::{Client, prelude::Config};

use crate::server::Server;

mod protocol;
mod client;
mod server;

// this is some of the worst code I've written, please don't look at it

#[tokio::main]
async fn main() {
        let config = Config {
            nickname: Some("ISLAND1".to_owned()),
            server: Some("mondecitronne.com".to_owned()),
            channels: vec!["#a".to_owned()],
            port: Some(6697),
            ..Default::default()
        };

        let client = Client::from_config(config).await.unwrap();

        let mut server = Server::new(client, "ISLAND1", Path::new("/home/lily/ISLAND1"));

        server.event_loop().await;
}
