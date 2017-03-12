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
                                        name  (commit.author().name().unwrap())
                                        email (commit.author().email().unwrap())
                                        image "http://en.gravatar.com/userimage/12799253/b889c035ec76c57ce679d12cbe01f2f4.png?s=64"
                                    }
                                    message (commit.message().unwrap())
                                    changes {
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
