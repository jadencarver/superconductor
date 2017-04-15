#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub focus: String,
    pub message: String,
    pub include: Vec<String>,
    pub property: Vec<Property>,
    pub diff: Vec<String>,
    pub save_update: Option<String>
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Property {
    pub name: String,
    pub value: String
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
