#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct State {
    focus: String,
    message: String,
    include: Vec<String>,
    property: Vec<Property>,
    diff: Vec<String>,
    save_update: Option<String>
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Property {
    name: String,
    value: String
}

impl State {
    pub fn blank() -> State {
        State {
            focus: String::new(),
            message: String::new(),
            include: vec![],
            diff: vec![],
            property: vec![],
            save_update: None
        }
    }
}
