use markup;

use state::State;
use task::Task;

use std::cell::RefCell;
use std::fs::File;
use std::io::prelude::*;
use std::env;
use termion::color;

extern crate git2;
use self::git2::Repository;
use self::git2::StatusOptions;
use self::git2::Delta;
use self::git2::BranchType;
use self::git2::{Diff, DiffDelta, DiffHunk, DiffLine, DiffBinary};
use git2::ObjectType;
use state::Filter;

extern crate md5;
extern crate chrono;
use self::chrono::{TimeZone, FixedOffset};

use yaml_rust::Yaml;

extern crate base64;

use self::base64::encode;

pub fn generate(state: Option<State>) -> String {
    let repo = match Repository::open_from_env() {
        Ok(repo) => repo,
        Err(_) => Repository::init(env::var("GIT_DIR").unwrap_or(String::from("."))).unwrap()
    };
    println!("Generating from repo: {:?}", repo.path());
    let config = repo.config().unwrap();
    
    // If there is no master branch, start the setup
    if repo.find_branch("master", BranchType::Local).is_err() {
        return GeneratedState {
            state: state
       }.to_string()
    }

    let head = repo.head().unwrap();
    let head_tree_obj = head.peel(ObjectType::Tree).unwrap();
    let head_tree = head_tree_obj.as_tree().unwrap();
    let changes = repo.diff_tree_to_index(Some(&head_tree), None, None).unwrap();

    let branches = repo.branches(Some(BranchType::Local)).unwrap().filter_map(|b|b.ok());
    let all_tasks: Vec<Task> = branches.filter_map(|(branch, _)| {
        let task = Task::from_ref(branch.get()).unwrap();
        if let Some(ref state) = state {
            if task.name != state.task { Some(task) } else { None }
        } else {
            if !branch.is_head() { Some(task) } else { None }
        }
    }).collect();
    
    // Set Filters
    let filter: Option<Filter> = match all_tasks.is_empty() {
        true => Some(Filter {
            name: String::from("Status"),
            value: String::from("Sprint")
        }),
        false => match state {
            Some(ref state) => match state.filter {
                Some(ref filter) if filter.name != "" => state.filter.clone(),
                _ => None,
            },
            None => None
        }
    };

    // Apply Filters
    let mut tasks: Vec<&Task> = match filter {
        Some(ref filter) => {
            let filter_name = Yaml::String(filter.name.clone());
            let filter_by_value = Yaml::String(filter.value.clone());
            all_tasks.iter().filter(|task| {
                match task.get(&repo, &filter_name) {
                    Some(ref value) if *value == filter_by_value => true,
                    Some(ref value) if *value == Yaml::Null => true,
                    None => true,
                    _ => false
                }
            }).collect()
        },
        None => all_tasks.iter().map(|t|t).collect()
    };

    tasks.sort_by(|a, b| {
        let ordinal = Yaml::from_str("Ordinal");
        let ord_a = a.get(&repo, &ordinal).unwrap_or(Yaml::Real(String::from("1.0"))).as_f64().unwrap_or(1.0);
        let ord_b = b.get(&repo, &ordinal).unwrap_or(Yaml::Real(String::from("1.0"))).as_f64().unwrap_or(1.0);
        ord_a.partial_cmp(&ord_b).unwrap()
    });

    let branch = match state.clone() {
        Some(state) => match repo.find_branch(&state.task, BranchType::Local) {
            Ok(branch) => branch.into_reference(),
            Err(_) => head
        },
        None => head
    };

    let task = Task::from_ref(&branch).unwrap();

    let mut revwalk = repo.revwalk().unwrap();
    revwalk.set_sorting(git2::Sort::REVERSE);
    revwalk.push(branch.target().unwrap()).unwrap();
    if branch.shorthand().unwrap() != "master" {
        revwalk.hide_ref("refs/heads/master").unwrap();
    }

    let mut status_opts = StatusOptions::new();
    status_opts.include_untracked(true);

    let payload = Payload {
        repo: &repo,
        state: state,
        branch: branch,
        filter: filter,
        tasks: tasks,
        task: task,
        //revwalk: revwalk,
        changes: changes,
        status_opts: status_opts,
        config: config
    }.to_string();
    println!("  {}Sent payload of size: {}{}", color::Fg(color::LightGreen), payload.len(), color::Fg(color::Reset));
    payload
}

fn diff<'a>(changes: &'a Diff) -> Vec<String> {
    let result = RefCell::new(vec![]);
    changes.foreach(&mut |delta: DiffDelta, _: f32| {
        if let Some(path) = delta.new_file().path() {
            result.borrow_mut().push(DiffLabel {path: path}.to_string());
        }
        true
    }, Some(&mut |delta: DiffDelta, binary: DiffBinary| {
        if binary.contains_data() {
            result.borrow_mut().push(DiffImage {
                data: String::from_utf8_lossy(&binary.new_file().data()).to_string()
            }.to_string());
        } else {
            let path = delta.new_file().path().unwrap();
            let mut file = File::open(path).unwrap();
            let mut contents = vec![];
            file.read_to_end(&mut contents).unwrap();
            result.borrow_mut().push(DiffImage64 {
                data: encode(&contents),
                path: path.to_str().unwrap()
            }.to_string());
        }
        true
    }), None, Some(&mut |_: DiffDelta, _: Option<DiffHunk>, line: DiffLine| {
        let class = match line.origin() {
            '+' | '>' => "add",
            '-' | '<' => "sub",
            'H' | 'F' => "meta",
            _ => ""
        };
        result.borrow_mut().push(DiffDiff {
            class: class,
            content: String::from_utf8_lossy(line.content()).to_string()
        }.to_string());
        true
    })).unwrap();
    result.into_inner()
}

markup::define! {
    Payload<'a>(repo: &'a git2::Repository, state: Option<State>, config: git2::Config, branch: git2::Reference<'a>, filter: Option<Filter>, tasks: Vec<&'a Task>, task: Task, changes: git2::Diff<'a>, status_opts: StatusOptions) {
        state {
            @if let Some(commit) = state.clone() {
                focus {{ commit.focus }}
            }
            user {
                name  {{config.get_string("user.name" ).unwrap_or(String::from("Unknown"))}}
                email {{config.get_string("user.email").unwrap_or(String::from("root@localhost"))}}
            }
            @if let Some(state) = state.clone() {
                message {{state.message}}
                @if state.property.len() == 0 {
                    @if let Ok(task) = Task::from_ref(&branch) {
                        {PayloadTask { repo: &repo, task: &task, properties: task.properties(&repo), changes: vec![] }}
                    }
                } else {
                    task {
                        name {{state.task}}
                        @for property in state.property {
                            property {
                                name {{property.name}}
                                value {{property.value}}
                            }
                        }
                    }
                }
            } else {
                @if let Ok(task) = Task::from_ref(&branch) {
                    {PayloadTask { repo: &repo, task: &task, properties: task.properties(&repo), changes: vec![] }}
                }
            }
            tasks {
                @if let Some(filter) = {filter} {
                    @if filter.name != "" {
                        filter {
                            name {{filter.name}}
                            value {{filter.value}}
                        }
                    }
                }
                @for task in {tasks} {
                    {PayloadTask { repo: &repo, task: &task, properties: task.properties(&repo), changes: vec![] }}
                }
            }
            log {
                //@for (_, rev) in revwalk.enumerate() {
                //    @if let Ok(commit) = repo.find_commit(rev.unwrap()) {
                //        commit {
                //            id {{ format!("{}", commit.id()) }}
                //            timestamp {{ commit.time().seconds() }}
                //            localtime {{ FixedOffset::east(commit.time().offset_minutes()*60).timestamp(commit.time().seconds(), 0).to_rfc3339() }}
                //            user {
                //                @if let Some(author_name) = commit.author().name() {
                //                    name {{ author_name }}
                //                    @if let Some(author_email) = commit.author().email() {
                //                        email {{ author_email.trim() }}
                //                        image {{ format!("https://www.gravatar.com/avatar/{:x}?s=64", md5::compute(author_email.trim().to_lowercase())) }}
                //                    }
                //                }
                //            }
                //            @if let Some(mut message) = commit.message() {
                //                message {{ message.split("---\n").next().unwrap() }}
                //                @if let Ok(task) = Task::from_commit(&branch.shorthand().unwrap(), &commit) {
                //                    {PayloadTask { repo: &repo, task: &task, properties: vec![], changes: task.changes(&repo) }}
                //                }
                //            }
                //        }
                //    }
                //}
            }
            @if task.name == "master" {
                {Project {}}
            } else {
                {Properties {}}
            }
            changes {
                @if let Ok(delta) = changes.stats() {
                    @if delta.files_changed() + delta.insertions() + delta.deletions() > 0 {
                        statistics {
                            files {{ delta.files_changed() }}
                            insertions {{ delta.insertions() }}
                            deletions {{ delta.deletions() }}
                        }
                    }
                }
                //@for change in repo.statuses(Some(&mut status_opts)).unwrap().iter() {
                //    @if let Some(path) = change.path() {
                //        change[id={path.replace("/", "_").replace(".", "_").replace(" ", "_")}] {
                //            path {{path}}
                //            insertions {}
                //            deletions {}
                //            included {
                //                {match change.head_to_index().map(|d| d.status()).unwrap_or(Delta::Unreadable) {
                //                    Delta::Modified | Delta::Added | Delta::Deleted => "true",
                //                    _ => "false"
                //                }}
                //            }
                //            removal {
                //                {match change.head_to_index().map(|d| d.status()).unwrap_or(Delta::Unreadable) {
                //                    Delta::Deleted => "true",
                //                    _ => "false"
                //                }}
                //            }
                //        }
                //    }
                //}
            }
            diffs {
                @for change in diff(changes) {
                    {change}
                }
            }
        }
    }
}

markup::define! {
    GeneratedState(state: Option<State>) {
        state {
            setup { "1" }
            @if let Some(state) = {state} {
                message {{ state.message }}
                task {
                    name { "master" }
                    @for property in {state.property} {
                        property {
                            name {{ property.name }}
                            value {{ property.value }}
                        }
                    }
                }
            } else {
                task {
                    name { "master" }
                }
            }
            {Project {}}
        }
    }

    DiffImage(data: String) {
        img[src={format!("data:image/png,{}", data)}] {}
    }

    DiffImage64<'a>(data: String, path: &'a str) {
        img[src={format!("data:image/jpeg;base64,{}", data)}, alt={format!("{}", path)}] {}
    }

    DiffDiff(class: &'static str, content: String) {
        span[class={class}] {{ content }}
    }

    DiffLabel<'a>(path: &'a std::path::Path) {
        label {{ path.to_str().unwrap_or("[invalid]") }}
    }

    PayloadTask<'a>(repo: &'a Repository, task: &'a Task, changes: Vec<(String, Option<String>, String)>, properties: Vec<(String, Option<String>, String)>) {
        task {
            name {{ task.name }}
            timestamp {{ task.timestamp(&repo) }}
            @for (name, before, value) in {changes} {
                property {
                    name {{ name }}
                    @if let Some(before) = {before} {
                        before {{ before }}
                    }
                    value {{ value }}
                }
            }
        }
    }

    Properties {
        properties {
            property {
                name { "Status" }
                options {
                    option { "Sprint" }
                    option { "In Progress" }
                    option { "In Review" }
                    option { "Blocked" }
                    option { "Done" }
                }
            }
            property {
                name { "Ordinal" }
                value { "1.0" }
            }
            property {
                name { "Estimate" }
            }
            property {
                name { "Developer" }
                value { "Jaden Carver <jaden.carver@gmail.com>" }
                options {
                    option[value="Jaden Carver <jaden.carver@gmail.com>"] { "Jaden Carver" }
                    option[value="Bob Dole <bdole69@gmail.com>"] { "Bob Dole" }
                }
            }
            property {
                name { "Manager" }
                value { "Jaden Carver <jaden.carver@gmail.com>" }
                options {
                    option[value="Jaden Carver <jaden.carver@gmail.com>"] { "Jaden Carver" }
                    option[value="Bob Dole <bdole69@gmail.com>"] { "Bob Dole" }
                }
            }
            property {
                name { "Description" }
            }
        }
    }

    Project {
        properties {
            property {
                name { "Project" }
            }
            property {
                name { "Status" }
                options {
                    option { "Sprint" }
                    option { "In Progress" }
                    option { "In Review" }
                    option { "Blocked" }
                    option { "Done" }
                }
            }
            property {
                name { "Manager" }
                value { "Jaden Carver <jaden.carver@gmail.com>" }
                options {
                    option[value="Jaden Carver <jaden.carver@gmail.com>"] { "Jaden Carver" }
                    option[value="Bob Dole <bdole69@gmail.com>"] { "Bob Dole" }
                }
            }
            property {
                name { "Description" }
            }
        }
    }
}
