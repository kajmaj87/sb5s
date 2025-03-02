pub struct Core {
    pub agents: Vec<String>,
}

impl Core {
    pub fn new() -> Self {
        Self { agents: Vec::new() }
    }

    pub fn add_agent(&mut self, name: String) {
        println!("Agent {} added!", name);
        self.agents.push(name);
    }
    pub fn move_agent(&self, name: &str, location: &str) {
        println!("Moving agent {} to {}", name, location);
    }
    pub fn get_agents(&self) -> Vec<String> {
        self.agents.clone()
    }
}
