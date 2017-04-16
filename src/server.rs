use std::thread;
use std::thread::JoinHandle;
use std::path::Path;
use std::time::Duration;

use websocket::{WebSocketStream, Server};
use websocket::Message as WebMessage;
use websocket::Sender as WebSender;
use websocket::client::Sender as WebClientSender;
use websocket::Receiver as WebReceiver;
use websocket::message::Type as WebMessageType;
use websocket::server::Connection;
use websocket::header::WebSocketProtocol;

use git2::Repository;
use git2::ObjectType;
use git2::BranchType;

use payload;
use state::State;

extern crate fsevent;
use self::fsevent::Event as FsEvent;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;

use serde_xml as xml;
use yaml_rust::yaml::Hash;
use yaml_rust::{Yaml, YamlEmitter};

#[derive(Debug)]
enum NotifierMessage<'a> {
    WebMessage(WebMessage<'a>),
    FsEvent(FsEvent)
}

pub fn start() {
    let server = Server::bind("127.0.0.1:2794").unwrap();
    thread::spawn(move || {
        for connection in server {
            thread::spawn(move || connect(connection.unwrap()));
        }
    });
}

fn connect(connection: Connection<WebSocketStream, WebSocketStream>) {
    let request = connection.read_request().unwrap();
    let headers = request.headers.clone();
    request.validate().unwrap();
    let mut response = request.accept();
    println!("Connected");

    if let Some(&WebSocketProtocol(ref protocols)) = headers.get() {
        if protocols.contains(&("superconductor".to_string())) {
            response.headers.set(WebSocketProtocol(vec!["superconductor".to_string()]));
        }
    }

    let mut client = response.send().unwrap();
    let message = WebMessage::text(payload::generate(None));
    client.send_message(&message).unwrap();

    let (tx, rx) = channel::<NotifierMessage>();
    let (mut sender, mut receiver) = client.split();
    let notifier = thread::spawn(move || start_notifier(rx, sender));
    let txc = tx.clone();
    let monitor = thread::spawn(move || start_monitor(txc));
    for message in receiver.incoming_messages() {
        let message: WebMessage = message.unwrap_or(WebMessage::close());
        match message.opcode {
            WebMessageType::Close => {
                tx.send(NotifierMessage::WebMessage(message));
                break;
            },
            _ => tx.send(NotifierMessage::WebMessage(message))
        };
    }
    monitor.join();
    notifier.join();
}

fn start_monitor(tx: Sender<NotifierMessage>) {
    let (ftx, rx) = channel();
    let observer = thread::spawn(move || {
        println!("Monitoring");
        let fsevent = fsevent::FsEvent::new(ftx);
        fsevent.append_path(".");
        fsevent.observe();
    });
    loop {
        let event = rx.recv().unwrap();
        thread::sleep(Duration::from_millis(500));
        let mut events = vec![];
        events.push(event);
        while let Ok(aggregator) = rx.try_recv() {
            events.push(aggregator);
        }
        tx.send(NotifierMessage::FsEvent(events.pop().unwrap()));
    }
    observer.join();
}


fn start_notifier(rx: Receiver<NotifierMessage>, mut sender: WebClientSender<WebSocketStream>) {
    let mut last_state: Option<State> = None;
    loop {
        let value = rx.recv().unwrap();
        match value {
            NotifierMessage::WebMessage(message) => {
                match message.opcode {
                    WebMessageType::Close => {
                        break;
                    },
                    WebMessageType::Ping => {
                        let message = WebMessage::pong(message.payload);
                        sender.send_message(&message).unwrap();
                    },
                    _ => {
                        let payload = String::from_utf8_lossy(message.payload.as_ref());
                        println!("{}", payload);
                        let state: State = xml::from_str(&payload).unwrap();
                        println!("{:?}", state);

                        let repo = Repository::discover(".").unwrap();
                        let branch = repo.find_branch(&state.task, BranchType::Local);
                        let head = match branch {
                            Ok(branch) => branch.into_reference(),
                            _ => repo.head().unwrap()
                        };
                        let commit = head.peel(ObjectType::Commit).unwrap();
                        let mut index = repo.index().unwrap();

                        let to_remove = index.iter().fold(vec![], |mut acc, entry| {
                            let entry_path = String::from_utf8_lossy(entry.path.as_ref());
                            match state.include.iter().find(|i| i.as_ref() == entry_path) {
                                None => acc.push(entry_path.into_owned()),
                                _ => {}
                            };
                            acc
                        });

                        repo.reset_default(Some(&commit), to_remove.iter()).unwrap();
                        for change in state.clone().include {
                            let path = Path::new(&change);
                            index.add_path(path).unwrap();
                        }
                        index.write().unwrap();

                        if let Some(event) = state.save_update.clone() {
                            let author = repo.signature().unwrap();
                            let mut yaml = String::new();
                            {
                                // Constructing the properties YAML
                                let mut tasks = Hash::new();
                                let mut properties = Hash::new();
                                let mut emitter = YamlEmitter::new(&mut yaml);
                                for property in state.property {
                                    properties.insert(Yaml::String(property.name), Yaml::String(property.value));
                                }
                                tasks.insert(Yaml::String(String::from("BLBA-1234")), Yaml::Hash(properties));
                                emitter.dump(&Yaml::Hash(tasks)).unwrap();
                            }
                            let message = state.message + "\n" + &yaml;

                            index.read(false);
                            let tree_oid = index.write_tree().unwrap();
                            let tree = repo.find_tree(tree_oid).unwrap();
                            repo.commit(Some(&head.name().unwrap_or("HEAD")), &author, &author, &message, &tree, &[&commit.as_commit().unwrap()]);
                            let message = WebMessage::text(payload::generate(None));
                            sender.send_message(&message).unwrap();
                            last_state = None;
                        } else {
                            last_state = Some(state);
                        }
                        let message = WebMessage::text(payload::generate(last_state.clone()));
                        sender.send_message(&message).unwrap();
                    }
                }
            },
            NotifierMessage::FsEvent(event) => {
                let repo = Repository::discover(".").unwrap();
                let message = WebMessage::text(payload::generate(last_state.clone()));
                sender.send_message(&message).unwrap();
            }
        }
    }
}
