use std;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use termion::color;
use termion::clear;
use termion::cursor;
use rand;

use state::generate;
use state::State;

use std::net::TcpListener;
use tungstenite::server::accept;
use tungstenite::protocol::Message as WebMessage;

extern crate fsevent;
//use self::fsevent::{ITEM_MODIFIED, ITEM_CREATED, ITEM_REMOVED};
use self::fsevent::Event as FsEvent;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;

use git2::Repository;

use serde_xml as xml;

const HEARTBEAT: u64 = 250;

#[derive(Debug)]
enum NotifierMessage {
    WebMessage(WebMessage),
    FsEvent(FsEvent)
}

pub fn start(port: i32) -> Result<TcpListener, std::io::Error> {
    let host = format!("127.0.0.1:{}", port);
    TcpListener::bind(host)
}

pub fn connect(stream: std::net::TcpStream) {
    let connection = Arc::new(accept(stream).unwrap());
    println!("Connected");

    let (tx, rx) = channel::<NotifierMessage>();
    let c = connection.clone();
    let notifier = thread::spawn(move || start_notifier(rx, c));

    let message = WebMessage::text(generate(None));
    tx.send(NotifierMessage::WebMessage(message)).unwrap();

    let monitor_tx = tx.clone();
    let monitor = thread::spawn(move || start_monitor(monitor_tx));

    let updater_tx = tx.clone();
    let updater = thread::spawn(move || start_updater(updater_tx));

    for message in connection.read_message() {
        //let message: WebMessage = message.unwrap_or(WebMessage::close());
        //match message.opcode {
            //WebMessage::Close => {
            //    tx.send(NotifierMessage::WebMessage(message)).unwrap();
            //    break;
            //},
            //_ =>
                tx.send(NotifierMessage::WebMessage(message));
        //}.unwrap();
    }

    monitor.join().unwrap();
    notifier.join().unwrap();
    updater.join().unwrap();
}

fn start_monitor(tx: Sender<NotifierMessage>) {
    let (ftx, rx) = channel();
    let observer = thread::spawn(move || {
        println!("Monitoring");
        let fsevent = fsevent::FsEvent::new(vec![".".to_string()]);
        fsevent.observe(ftx);
    });
    loop {
        let event = rx.recv().unwrap();
        thread::sleep(Duration::from_millis(HEARTBEAT));
        let mut changes = vec![event];
        loop {
            if let Ok(event) = rx.try_recv() {
                //if (event.flag.contains(ITEM_MODIFIED) || event.flag.contains(ITEM_CREATED) || event.flag.contains(ITEM_REMOVED)) && (!event.path.contains(".git") || !event.path.contains(".lock")) {
                    changes.push(event);
                //}
            } else {
                break;
            }
        }
        if !changes.is_empty() {
            if tx.send(NotifierMessage::FsEvent(changes.pop().unwrap())).is_err() {
                break;
            };
        }
    }
    observer.join().unwrap();
}


fn start_notifier(rx: Receiver<NotifierMessage>, mut sender: Arc<tungstenite::WebSocket<std::net::TcpStream>>) {
    let mut last_state: Option<State> = None;
    let mut rng = rand::thread_rng();
    loop {
        let event = rx.recv().unwrap();
        match event {
            NotifierMessage::WebMessage(message) => {
                if let WebMessage::Text(payload) = message {
                    //println!("\n\n{}{}{}{}{}", clear::All, cursor::Goto(1, 1), color::Fg(color::White), payload, color::Fg(color::Reset));
                    let mut state: State = xml::from_str(&payload).unwrap_or(last_state.clone().unwrap_or(State::blank()));

                    last_state = state.apply(last_state, &mut rng).unwrap();
                    let message = WebMessage::text(generate(Some(state)));
                    sender.write_message(message).unwrap();

                    //thread::sleep(Duration::from_millis(HEARTBEAT));
                    flush(&rx);
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
