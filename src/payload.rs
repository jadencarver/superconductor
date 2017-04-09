use State;
use project;

extern crate git2;
use self::git2::Repository;
use self::git2::Reference;
use self::git2::StatusOptions;
use self::git2::Delta;
use git2::ObjectType;

extern crate md5;

extern crate yaml_rust;
use self::yaml_rust::YamlLoader;

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
            @if let Some(commit) = previous_commit.clone() {
                message (commit.message)
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
                @if let Some(commit) = previous_commit.clone() {
                    @for path in commit.diff.iter() {
                        diff { "I_AM_DIFF" }
                    }
                }
            }
        }
    }.into_string();
    payload
}