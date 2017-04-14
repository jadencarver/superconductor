use State;
use project;

use std::cell::RefCell;

extern crate git2;
use self::git2::Repository;
use self::git2::Reference;
use self::git2::StatusOptions;
use self::git2::Delta;
use self::git2::{Diff, DiffFormat, DiffDelta, DiffHunk, DiffLine};
use git2::ObjectType;

extern crate md5;
extern crate chrono;
use maud::PreEscaped;
use self::chrono::{TimeZone, FixedOffset};

use yaml_rust::{Yaml, YamlLoader};

pub fn generate(previous_commit: Option<State>) -> String {
    let current = project::current();
    let repo = Repository::discover(".").unwrap();
    let mut revwalk = repo.revwalk().unwrap();
    revwalk.set_sorting(git2::SORT_REVERSE);
    revwalk.push_head().unwrap();

    let head = repo.head().unwrap().peel(ObjectType::Tree).unwrap();
    let head_tree = head.as_tree().unwrap();
    let changes = repo.diff_tree_to_index(Some(&head_tree), None, None).unwrap();

    let mut status_opts = StatusOptions::new();
    status_opts.include_untracked(true);

    let payload = html! {
        state {
            @if let Some(commit) = previous_commit.clone() {
                focus (commit.focus)
            }
            user {
                name  (current.user.name)
                email (current.user.email)
            }
            task {
                id (current.task.id)
            }
            @if let Ok(branches) = repo.branches(None) {
                tasks {
                    @for (branch, branch_type) in branches.map(|b|b.unwrap()) {
                        task {
                            name (branch.name().ok().unwrap().unwrap())
                        }
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
                                @if let Some(yaml) = message.next() {
                                    @for (task, values) in YamlLoader::load_from_str(yaml).unwrap()[0].as_hash().unwrap() {
                                        task {
                                            name (task.as_str().unwrap_or("None"))
                                            @for (name, value) in values.as_hash().unwrap() {
                                                property {
                                                    name (name.as_str().unwrap_or("[]"))
                                                    @for parent in commit.parents() {
                                                        @let mut propwalk = repo.revwalk().unwrap() {
                                                            @if let Ok(_) = propwalk.push(parent.id()) {
                                                                @for proprev in propwalk {
                                                                    @let propcommit = repo.find_commit(proprev.unwrap()).unwrap() {
                                                                        @let mut propmessages = propcommit.message().unwrap().split("---\n") {
                                                                            @if let Some(propmessage) = propmessages.next() {
                                                                                @if let Some(yaml) = propmessages.next() {
                                                                                    @for (proptask, propvalues) in YamlLoader::load_from_str(yaml).unwrap()[0].as_hash().unwrap() {
                                                                                        @if proptask == proptask {
                                                                                            @for (propname, propvalue) in propvalues.as_hash().unwrap() {
                                                                                                @if propname == name {
                                                                                                    @if let Some(propbefore) = propvalue.as_str() {
                                                                                                        before (propbefore)
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
                                                            }
                                                        }
                                                    }
                                                    after {
                                                        @match value {
                                                            &Yaml::Integer(int) => (int),
                                                            &Yaml::String(ref string) => (string),
                                                            &Yaml::Boolean(b) => (b),
                                                            _ => ("[unknown]")
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
                }
            }
            @if let Some(commit) = previous_commit.clone() {
                properties {
                    @for property in commit.property {
                        property {
                            name (property.name)
                            value (property.value)
                            options {
                                option "Sprint"
                                option "In Progress"
                                option "In Review"
                                option "Blocked"
                                option "Done"
                            }
                        }
                    }
                }
                message (commit.message)
            } @else {
                properties {
                    property {
                        name "Status"
                        value "Done"
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
                        value "5"
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
                        value ""
                    }
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
                        change id=(path.replace("/", "_").replace(".", "_")) {
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
    }, None, None, Some(&mut |delta: DiffDelta, hunk: Option<DiffHunk>, line: DiffLine| {
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
