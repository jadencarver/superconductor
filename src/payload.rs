use Commit;
use project;

extern crate git2;
use self::git2::Repository;
use self::git2::Reference;
use self::git2::StatusOptions;
use self::git2::Delta;

extern crate md5;

extern crate yaml_rust;
use self::yaml_rust::YamlLoader;

pub fn generate(previous_commit: Option<Commit>) -> String {
    let current = project::current();
    let repo = Repository::discover(".").unwrap();
    let mut revwalk = repo.revwalk().unwrap();
    revwalk.set_sorting(git2::SORT_REVERSE);
    revwalk.push_head().unwrap();
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
            @if let Some(commit) = previous_commit.clone() {
                message (commit.message)
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
