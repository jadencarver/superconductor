use std::thread;
use std::path::Path;
use std::cell::RefCell;

use websocket::{Server, Message, Sender, Receiver};
use websocket::message::Type;
use websocket::header::WebSocketProtocol;

extern crate md5;

extern crate git2;
use self::git2::Repository;
use self::git2::Reference;
use self::git2::Statuses;
use self::git2::StatusOptions;
use self::git2::Delta;
use self::git2::ObjectType;

extern crate yaml_rust;
use self::yaml_rust::YamlLoader;

use project;

use serde_xml as xml;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Commit {
    focus: String,
    message: String,
    include: Vec<String>,
    save_update: Option<String>
}

impl Commit {
}

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
                let message = Message::text(generate_payload(None));
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

                            repo.reset_default(Some(&head), to_remove.iter());
                            for change in commit.clone().include {
                                let path = Path::new(&change);
                                index.add_path(path);
                            }
                            index.write().unwrap();
                            let message = Message::text(generate_payload(Some(commit)));
                            sender.send_message(&message).unwrap();
                        }
                    }
                }
            });
        }
    });
}

fn generate_payload(previous_commit: Option<Commit>) -> String {
    let current = project::current();
    let repo = Repository::discover(".").unwrap();
    let mut revwalk = repo.revwalk().unwrap();
    revwalk.set_sorting(git2::SORT_REVERSE);
    revwalk.push_head();
    let mut status_opts = StatusOptions::new();
    status_opts.include_untracked(true);

    let payload = html! {
        state {
            @if let Some(commit) = previous_commit {
                focus (commit.focus)
            }
            user {
                name  (current.user.name)
                email (current.user.email)
            }
            task {
                id (current.task.id)
            }
            log {
                @for rev in revwalk {
                    @let commit = repo.find_commit(rev.unwrap()).unwrap() {
                        commit {
                            id (commit.id())
                            user {
                                @let author = commit.author() {
                                    name  (author.name().unwrap())
                                        @let email = author.email().unwrap().trim() {
                                            email (email)
                                            image (format!("https://www.gravatar.com/avatar/{:x}?s=64", md5::compute(email.to_lowercase())))
                                        }
                                }
                            }
                            @let mut message = commit.message().unwrap().split("\n---") {
                                message (message.next().unwrap())
                                @if let Some(yaml) = message.next() {
                                    @for (objective, values) in YamlLoader::load_from_str(yaml).unwrap()[0].as_hash().unwrap() {
                                        objective {
                                            name (objective.as_str().unwrap_or("None"))
                                            @for (name, value) in values.as_hash().unwrap() {
                                                property {
                                                    name (name.as_str().unwrap_or("None"))
                                                    after (value.as_str().unwrap_or("None"))
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            changes {
                @for change in repo.statuses(Some(&mut status_opts)).unwrap().iter() {
                    change {
                        path (change.path().unwrap())
                        included @match change.head_to_index().map(|d| d.status()).unwrap_or(Delta::Unreadable) {
                            Delta::Modified | Delta::Added | Delta::Deleted => "true",
                            _ => "false"
                        }
                        removal @match change.head_to_index().map(|d| d.status()).unwrap_or(Delta::Unreadable) {
                            Delta::Deleted => "true",
                            _ => "false"
                        }
                    }
                }
            }
        }
    }.into_string();
    payload
}
