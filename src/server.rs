use std;
use std::thread;
use std::time::Duration;
use termion::color;
use termion::clear;
use termion::cursor;
use rand;

use websocket::{WebSocketStream, Server};
use websocket::Message as WebMessage;
use websocket::Sender as WebSender;
use websocket::client::Sender as WebClientSender;
use websocket::Receiver as WebReceiver;
use websocket::message::Type as WebMessageType;
use websocket::server::Connection;
use websocket::header::WebSocketProtocol;

use state::generate;
use state::State;

extern crate fsevent;
use self::fsevent::{ITEM_MODIFIED, ITEM_CREATED, ITEM_REMOVED};
use self::fsevent::Event as FsEvent;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;

use git2::Repository;

use serde_xml as xml;

const HEARTBEAT: u64 = 250;

#[derive(Debug)]
enum NotifierMessage<'a> {
    WebMessage(WebMessage<'a>),
    FsEvent(FsEvent)
}

pub fn start(port: i32) -> Result<Server<'static>, std::io::Error> {
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
    let message = WebMessage::text(generate(None));
    client.send_message(&message).unwrap();

    let (tx, rx) = channel::<NotifierMessage>();
    let (sender, mut receiver) = client.split();
    let notifier = thread::spawn(move || start_notifier(rx, sender));
    let monitor_tx = tx.clone();
    let monitor = thread::spawn(move || start_monitor(monitor_tx));
    let updater_tx = tx.clone();
    let updater = thread::spawn(move || start_updater(updater_tx));
    for message in receiver.incoming_messages() {
        let message: WebMessage = message.unwrap_or(WebMessage::close());
        match message.opcode {
            WebMessageType::Close => {
                tx.send(NotifierMessage::WebMessage(message)).unwrap();
                break;
            },
            _ => tx.send(NotifierMessage::WebMessage(message))
        }.unwrap();
    }
    monitor.join().unwrap();
    notifier.join().unwrap();
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
        thread::sleep(Duration::from_millis(HEARTBEAT));
        let mut changes = vec![event];
        loop {
            if let Ok(event) = rx.try_recv() {
                if (event.flag.contains(ITEM_MODIFIED) || event.flag.contains(ITEM_CREATED) || event.flag.contains(ITEM_REMOVED)) && (!event.path.contains(".git") || !event.path.contains(".lock")) {
                    changes.push(event);
                }
            } else {
                break;
            }
        }
        if !changes.is_empty() {
            tx.send(NotifierMessage::FsEvent(changes.pop().unwrap())).unwrap();
        }
    }
    observer.join().unwrap();
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
                        //println!("\n\n{}{}{}{}{}", clear::All, cursor::Goto(1, 1), color::Fg(color::White), payload, color::Fg(color::Reset));
                        let mut state: State = xml::from_str(&payload).unwrap_or(last_state.clone().unwrap_or(State::blank()));

                        last_state = state.apply(last_state, &mut rng).unwrap();
                        let message = WebMessage::text(generate(Some(state)));
                        sender.send_message(&message).unwrap();

                        thread::sleep(Duration::from_millis(HEARTBEAT));
                        flush(&rx);
                    }
                }
            },
            NotifierMessage::FsEvent(_event) => {
                //let message = WebMessage::text("submit");
                //sender.send_message(&message).unwrap();
            }
        }
    }
}

fn start_updater(tx: Sender<NotifierMessage>) {
    loop {
        thread::sleep(Duration::from_secs(300));
        let repo = Repository::open_from_env().unwrap_or(Repository::init(".").unwrap());
        if let Ok(mut origin) = repo.find_remote("origin") {
            println!("Updating from remote...");
            origin.fetch(&["master"], None, None);
        };
    }
}

fn flush<T>(channel: &Receiver<T>) {
    let mut flushed: i32 = 0;
    while channel.try_recv().is_ok() {
        flushed += 1;
    }
    let plural = if flushed != 1 { 's' } else { ' ' };
    println!("  {}Flushed {} event{}{}", color::Fg(color::Yellow), flushed, plural, color::Fg(color::Reset));
}
