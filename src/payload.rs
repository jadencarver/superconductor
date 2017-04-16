use state::State;
use task::Task;

use std::cell::RefCell;
use std::fs::File;
use std::io::prelude::*;

extern crate git2;
use self::git2::Repository;
use self::git2::Reference;
use self::git2::StatusOptions;
use self::git2::Delta;
use self::git2::BranchType;
use self::git2::{Diff, DiffFormat, DiffDelta, DiffHunk, DiffLine, DiffBinary};
use git2::ObjectType;

extern crate md5;
extern crate chrono;
use maud::PreEscaped;
use self::chrono::{TimeZone, FixedOffset};

use yaml_rust::{Yaml, YamlLoader};

extern crate base64;

use self::base64::{encode, decode};

pub fn generate(state: Option<State>) -> String {
    let repo = Repository::discover(".").unwrap();
    let config = repo.config().unwrap();
    let mut revwalk = repo.revwalk().unwrap();
    revwalk.set_sorting(git2::SORT_REVERSE);
    revwalk.push_head().unwrap();

    let head = repo.head().unwrap();
    let head_tree_obj = head.peel(ObjectType::Tree).unwrap();
    let head_tree = head_tree_obj.as_tree().unwrap();
    let head_commit_obj = head.peel(ObjectType::Commit).unwrap();
    let head_commit = head_commit_obj.as_commit().unwrap();
    let changes = repo.diff_tree_to_index(Some(&head_tree), None, None).unwrap();

    let mut status_opts = StatusOptions::new();
    status_opts.include_untracked(true);

    let payload = html! {
        state {
            @if let Some(commit) = state.clone() {
                focus (commit.focus)
            }
            user {
                name  (config.get_string("user.name" ).unwrap_or(String::from("Unknown")))
                email (config.get_string("user.email").unwrap_or(String::from("root@localhost")))
            }
            @if let Some(state) = state {
                message (state.message)
                task {
                    name "undefined"
                    @for property in state.property {
                        property {
                            name (property.name)
                            value (property.value)
                        }
                    }
                }
            } @else {
                @let mut message = head_commit.message().unwrap().split("---\n") {
                    message (message.next().unwrap())
                    @for task in Task::from_commit(&repo, &head_commit, message.next().unwrap_or("")) {
                        (render_task(&task, task.changes(&repo, &head_commit, false)))
                    }
                }
            }
            @if let Ok(branches) = repo.branches(None) {
                tasks {
                    @for (branch, branch_type) in branches.map(|b|b.unwrap()).filter(|&(ref b, t)| !b.is_head()) {
                        @if let Some(commit) = branch.get().peel(ObjectType::Commit).unwrap().as_commit() {
                            @let mut message = commit.message().unwrap().split("---\n") {
                                @if let Some(_) = message.next() {
                                    @for task in Task::from_commit(&repo, &commit, message.next().unwrap_or("")) {
                                        (render_task(&task, task.changes(&repo, &commit, false)))
                                    }
                                }
                            }
                        }
                        //task {
                        //    name (branch.name().ok().unwrap().unwrap())
                        //    type @match branch_type {
                        //        BranchType::Remote => "remote",
                        //        BranchType::Local => "local",
                        //        _ => ""
                        //    }
                        //}
                    }
                }
            }
            log {
                @for rev in revwalk {
                    @let commit = repo.find_commit(rev.unwrap()).unwrap() {
                        commit {
                            id (commit.id())
                            @let time = commit.time() {
                                timestamp (time.seconds())
                                localtime (FixedOffset::east(time.offset_minutes()*60).timestamp(time.seconds(), 0).to_rfc3339())
                            }
                            user {
                                @let author = commit.author() {
                                    name (author.name().unwrap())
                                    @let email = author.email().unwrap().trim() {
                                        email (email)
                                        image (format!("https://www.gravatar.com/avatar/{:x}?s=64", md5::compute(email.to_lowercase())))
                                    }
                                }
                            }

                            @let mut message = commit.message().unwrap().split("---\n") {
                                message (message.next().unwrap())
                                @for task in Task::from_commit(&repo, &commit, message.next().unwrap_or("")) {
                                    (render_task(&task, task.changes(&repo, &commit, true)))
                                }
                            }
                        }
                    }
                }
            }
            properties {
                property {
                    name "Status"
                    options {
                        option "Sprint"
                        option "In Progress"
                        option "In Review"
                        option "Blocked"
                        option "Done"
                    }
                }
                property {
                    name "Estimate"
                }
                property {
                    name "Developer"
                    value "Jaden Carver <jaden.carver@gmail.com>"
                    options {
                        option value="Jaden Carver <jaden.carver@gmail.com>" "Jaden Carver"
                        option value="Bob Dole <bdole69@gmail.com>" "Bob Dole"
                    }
                }
                property {
                    name "Description"
                }
            }
            changes {
                @if let Ok(delta) = changes.stats() {
                    statistics {
                        files (delta.files_changed())
                        insertions (delta.insertions())
                        deletions (delta.deletions())
                    }
                }
                @for change in repo.statuses(Some(&mut status_opts)).unwrap().iter() {
                    @let path = change.path().unwrap() {
                        change id=(path.replace("/", "_").replace(".", "_").replace(" ", "_")) {
                            path (path)
                            insertions {}
                            deletions {}
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
            diffs {
                @for change in diff(changes) {
                    (change)
                }
            }
        }
    }.into_string();
    payload
}

fn diff(changes: Diff) -> Vec<PreEscaped<String>> {
    let mut result = RefCell::new(vec![]);
    changes.foreach(&mut |delta: DiffDelta, i: f32| {
        result.borrow_mut().push(html!(
            @if let Some(path) = delta.new_file().path() {
                label (path.to_str().unwrap_or("[invalid]"))
            }
        ));
        true
    }, Some(&mut |delta: DiffDelta, binary: DiffBinary| {
        if binary.contains_data() {
            result.borrow_mut().push(html!(
                img src=(format!("data:image/jpeg,{}", (String::from_utf8_lossy(&binary.new_file().data())))) {}
            ));
        } else {
            let path = delta.new_file().path().unwrap();
            let mut file = File::open(path).unwrap();
            let mut contents = vec![];
            file.read_to_end(&mut contents).unwrap();
            result.borrow_mut().push(html!(
                img src=(format!("data:image/jpeg;base64,{}", encode(&contents))) alt=(format!("{}", path.to_str().unwrap())) {}
            ));
        }
        true
    }), None, Some(&mut |delta: DiffDelta, hunk: Option<DiffHunk>, line: DiffLine| {
        let class = match line.origin() {
            '+' | '>' => "add",
            '-' | '<' => "sub",
            'H' | 'F' => "meta",
            _ => ""
        };
        result.borrow_mut().push(html!(
            span class=(class) (String::from_utf8_lossy(&line.content()))
        ));
        true
    }));
    result.into_inner()
}

fn render_task(task: &Task, changes: Vec<(String, Option<String>, String)>) -> PreEscaped<String> {
    html!(task {
        name (task.name)
        @for (name, before, value) in changes {
            property {
                name (name)
                @if let Some(before) = before {
                    before (before)
                }
                value (value)
            }
        }
    })
}
