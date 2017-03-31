use std::thread;
use std::path::Path;

use websocket::{Server, Message, Sender, Receiver};
use websocket::message::Type;
use websocket::header::WebSocketProtocol;

extern crate git2;
use self::git2::Repository;
use self::git2::ObjectType;

use payload;
use Commit;

use serde_xml as xml;

pub fn start() {
    let server = Server::bind("127.0.0.1:2794").unwrap();
    thread::spawn(move || {
        for connection in server {
            thread::spawn(move || {
                let request = connection.unwrap().read_request().unwrap();
                let headers = request.headers.clone();
                request.validate().unwrap();
                let mut response = request.accept();
                println!("Connected");

                if let Some(&WebSocketProtocol(ref protocols)) = headers.get() {
                    if protocols.contains(&("rust-websocket".to_string())) {
                        response.headers.set(WebSocketProtocol(vec!["superconductor".to_string()]));
                    }
                }

                let mut client = response.send().unwrap();
                let message = Message::text(payload::generate(None));
                client.send_message(&message).unwrap();

                let (mut sender, mut receiver) = client.split();
                for message in receiver.incoming_messages() {
                    let message: Message = message.unwrap_or(Message::close());
                    match message.opcode {
                        Type::Close => {
                            let message = Message::close();
                            sender.send_message(&message).unwrap();
                            println!("Client disconnected");
                            return;
                        },
                        Type::Ping => {
                            let message = Message::pong(message.payload);
                            sender.send_message(&message).unwrap();
                        },
                        _ => {
                            let payload = String::from_utf8_lossy(message.payload.as_ref());
                            println!("{:?}", payload);
                            let commit: Commit = xml::from_str(&payload).unwrap();
                            println!("{:?}", commit);
                            let repo = Repository::discover(".").unwrap();
                            let head = repo.head().unwrap().peel(ObjectType::Commit).unwrap();
                            let mut index = repo.index().unwrap();

                            let to_remove = index.iter().fold(vec![], |mut acc, entry| {
                                let entry_path = String::from_utf8_lossy(entry.path.as_ref());
                                match commit.include.iter().find(|i| i.as_ref() == entry_path) {
                                    None => acc.push(entry_path.into_owned()),
                                    _ => {}
                                };
                                acc
                            });

                            repo.reset_default(Some(&head), to_remove.iter()).unwrap();
                            for change in commit.clone().include {
                                let path = Path::new(&change);
                                index.add_path(path);
                            }
                            index.write().unwrap();
                            let message = Message::text(payload::generate(Some(commit)));
                            sender.send_message(&message).unwrap();
                        }
                    }
                }
            });
        }
    });
}

