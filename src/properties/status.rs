struct Select {
    name: String,
    options: Vec<String>,
    selected: Option<usize>
}

impl Select {
    fn new(name: &str) -> Select {
        Select {
            name: String::from(str),
            options: vec![],
            selected: None
        }
    }
}
