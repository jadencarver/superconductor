use std::thread;
use std::thread::JoinHandle;
use std::path::Path;

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

use payload;
use State;

extern crate fsevent;
use self::fsevent::Event as FsEvent;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;

use serde_xml as xml;

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
        if protocols.contains(&("rust-websocket".to_string())) {
            response.headers.set(WebSocketProtocol(vec!["superconductor".to_string()]));
        }
    }

    let mut client = response.send().unwrap();
    let message = WebMessage::text(payload::generate(None));
    client.send_message(&message).unwrap();

    let (tx, rx) = channel::<NotifierMessage>();
    let (mut sender, mut receiver) = client.split();
    thread::spawn(move || start_notifier(rx, sender));
    let txc = tx.clone();
    thread::spawn(move || start_monitor(txc));
    for message in receiver.incoming_messages() {
        let message: WebMessage = message.unwrap_or(WebMessage::close());
        tx.send(NotifierMessage::WebMessage(message));
    }
}

fn start_notifier(rx: Receiver<NotifierMessage>, mut sender: WebClientSender<WebSocketStream>) {
    loop {
        let value = rx.recv().unwrap();
        println!("{:?}", value);
        match value {
            NotifierMessage::WebMessage(message) => {
                match message.opcode {
                    WebMessageType::Close => {
                        let message = WebMessage::close();
                        //sender.send_message(&message).unwrap();
                        println!("Client disconnected");
                        return;
                    },
                    WebMessageType::Ping => {
                        let message = WebMessage::pong(message.payload);
                        //sender.send_message(&message).unwrap();
                    },
                    _ => {
                        let payload = String::from_utf8_lossy(message.payload.as_ref());
                        println!("{:?}", payload);
                        let mut state: State = xml::from_str(&payload).unwrap();
                        println!("{:?}", state);

                        let repo = Repository::discover(".").unwrap();
                        let head = repo.head().unwrap().peel(ObjectType::Commit).unwrap();
                        let mut index = repo.index().unwrap();

                        let to_remove = index.iter().fold(vec![], |mut acc, entry| {
                            let entry_path = String::from_utf8_lossy(entry.path.as_ref());
                            match state.include.iter().find(|i| i.as_ref() == entry_path) {
                                None => acc.push(entry_path.into_owned()),
                                _ => {}
                            };
                            acc
                        });

                        repo.reset_default(Some(&head), to_remove.iter()).unwrap();
                        for change in state.clone().include {
                            let path = Path::new(&change);
                            index.add_path(path).unwrap();
                        }
                        index.write().unwrap();

                        if let Some(event) = state.save_update.clone() {
                            let author = repo.signature().unwrap();
                            index.read(false);
                            let tree_oid = index.write_tree().unwrap();
                            let tree = repo.find_tree(tree_oid).unwrap();
                            repo.commit(Some("HEAD"), &author, &author, &state.message, &tree, &[&head.as_commit().unwrap()]);
                            let message = WebMessage::text(payload::generate(None));
                            sender.send_message(&message).unwrap();
                        } else {
                            let message = WebMessage::text(payload::generate(Some(state)));
                            sender.send_message(&message).unwrap();
                        }
                    }
                }
            },
            NotifierMessage::FsEvent(event) => {}
        }
    }
}

fn start_monitor(tx: Sender<NotifierMessage>) {
    let (ftx, rx) = channel();
    thread::spawn(move || {
        println!("Monitoring");
        let fsevent = fsevent::FsEvent::new(ftx);
        fsevent.append_path(".");
        fsevent.observe();
    });
    loop {
        let event = rx.recv().unwrap();
        tx.send(NotifierMessage::FsEvent(event));
    }
}

//for message in receiver.incoming_messages() {

//}
