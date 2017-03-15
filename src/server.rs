use websocket::{Server, Message, Sender, Receiver};
use websocket::message::Type;
use websocket::header::WebSocketProtocol;
use std::thread;

extern crate git2;
use self::git2::Repository;
use self::git2::Reference;
use self::git2::Statuses;
use self::git2::StatusOptions;
use self::git2::Delta;

extern crate md5;

extern crate yaml_rust;
use self::yaml_rust::YamlLoader;

use project;

pub fn start() {
    let server = Server::bind("127.0.0.1:2794").unwrap();
    thread::spawn(move || {
        for connection in server {
            let current = project::current();
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

            let repo = Repository::open("/Users/jadencarver/dev/superconductor").unwrap();
            let mut revwalk = repo.revwalk().unwrap();
            revwalk.set_sorting(git2::SORT_REVERSE);
            revwalk.push_head();
            let mut status_opts = StatusOptions::new();
            status_opts.include_untracked(true);

            let mut client = response.send().unwrap();
            let message: Message = Message::text(html! {
                state {
                    user {
                        name  (current.user.name)
                        email (current.user.email)
                    }
                    task {
                        id (current.task.id)
                    }
                    history {
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
            }.into_string());
            client.send_message(&message).unwrap();
        }
    });
}
