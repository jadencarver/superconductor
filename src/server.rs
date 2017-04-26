use std;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use rand;

use websocket::{WebSocketStream, Server};
use websocket::Message as WebMessage;
use websocket::Sender as WebSender;
use websocket::client::Sender as WebClientSender;
use websocket::Receiver as WebReceiver;
use websocket::message::Type as WebMessageType;
use websocket::server::Connection;
use websocket::header::WebSocketProtocol;

use payload;
use state::State;

extern crate fsevent;
use self::fsevent::{ITEM_MODIFIED, ITEM_CREATED};
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

pub fn start(port: Option<i32>) -> Result<Server<'static>, std::io::Error> {
    let port = port.unwrap_or(2794);
    let host = format!("127.0.0.1:{}", port);
    Server::bind(host)
}

pub fn connect(connection: Connection<WebSocketStream, WebSocketStream>) {
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
        let mut changes = vec![];
        loop {
            if let Ok(event) = rx.try_recv() {
                if (event.flag.contains(ITEM_MODIFIED) || event.flag.contains(ITEM_CREATED)) && (!event.path.contains(".git") || !event.path.contains(".lock")) {
                    println!("Registered {:?}", event);
                    changes.push(event);
                } else {
                    println!("Ignored {:?}", event);
                }
            } else {
                break;
            }
        }
        if !changes.is_empty() {
            tx.send(NotifierMessage::FsEvent(changes.pop().unwrap()));
        }
    }
    observer.join();
}


fn start_notifier(rx: Receiver<NotifierMessage>, mut sender: WebClientSender<WebSocketStream>) {
    let mut last_state: Option<State> = None;
    let mut rng = rand::thread_rng();
    loop {
        let event = rx.recv().unwrap();
        match event {
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
                        let mut state: State = xml::from_str(&payload).unwrap();

                        last_state = state.apply(last_state, &mut rng).ok();

                        let message = WebMessage::text(payload::generate(Some(state)));
                        sender.send_message(&message).unwrap();
                    }
                }
            },
            NotifierMessage::FsEvent(event) => {
                let message = WebMessage::text(payload::generate(last_state.clone()));
                sender.send_message(&message).unwrap();
            }
        }
    }
}
