#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub task: String,
    pub focus: String,
    pub message: String,
    pub include: Vec<String>,
    pub property: Vec<Property>,
    pub diff: Vec<String>,
    pub save_update: Option<String>,
    pub new_task: Option<String>
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Property {
    pub name: String,
    pub value: String
}

impl State {
    pub fn blank() -> State {
        State {
            task: String::from("master"),
            focus: String::new(),
            message: String::new(),
            include: vec![],
            diff: vec![],
            property: vec![],
            save_update: None
        }
    }
    pub fn reset(&self, status: bool) -> State {
        let mut properties = vec![];
        if (status) {
            properties = self.property.iter().filter_map(|p| {
                if p.name == "Status" {
                    Some(p.clone())
                } else {
                    None
                }
            }).collect();
        }
        State {
            task: self.task.clone(),
            focus: self.focus.clone(),
            message: String::new(),
            include: vec![],
            property: properties,
            diff: vec![],
            save_update: None
        }
    }
}
