use std::fs::File;
use std::io::Read;
use std::path::Path;

use futures::prelude::*;
use irc::client::Client;
use irc::proto::Command;
use std::default::Default;
use std::env;

use crate::protocol::common::Message;
use crate::protocol::common::MessageType;

pub struct Server<'a> {
    name: String,
    directory: &'a Path,
    client: Client,
}

impl<'a> Server<'a> {
    pub fn new<S: AsRef<str>>(client: Client, name: S, directory: &'a Path) -> Self {
        Self {
            name: name.as_ref().to_string(),
            directory,
            client,
        }
    }

    pub async fn event_loop(&mut self) { // all of this needs error handling
        self.client.identify().unwrap(); // UNJUSTIFIED UNWRAP

        let mut stream = self.client.stream().unwrap(); //UNJUSTIFIED UNWRAP
        let sender = self.client.sender();

        while let Some(message) = stream.next().await.transpose().unwrap() {
            match message.command {
                Command::PRIVMSG(ref target, ref msg) => {
                    if msg.starts_with(r"\sea.") {
                        let message = Message::from_string(msg.to_string()).ok().unwrap(); //UNJUSTIFIED UNWRAP
                        if message.reciever() == self.name {
                            match message.message_type() {
                                MessageType::Sea0 { action, payload } => {
                                    if action == "req" {
                                        println!("{}", payload);
                                        let relative_path = Path::new(&payload);

                                        if !relative_path.is_absolute() {
                                            break;
                                        }
                                        
                                        let path_string = self.directory.to_str().unwrap().to_string() + &payload; // this is fucking terrible and a security vulnerability

                                        let absolute_path = Path::new(&path_string);

                                        println!("{}", absolute_path.to_str().unwrap());

                                        if absolute_path.is_file() {
                                            let mut file = File::open(absolute_path).unwrap();
                                            let mut contents = String::new();
                                            file.read_to_string(&mut contents).ok();
                                            contents = contents.replace("\n", r"\n");
                                            
                                            let responses = Message::make_resource_response(self.name.to_string(), message.sender(), contents);

                                            for response in responses {
                                                println!("{}", response);
                                                self.client.send(Command::PRIVMSG(target.to_string(), response.to_string())).ok();
                                            }
                                        }
                                    }
                                },
                                _ => (),
                            }
                        }
                    }
                }
                _ => (),
            }
        }
    }
}