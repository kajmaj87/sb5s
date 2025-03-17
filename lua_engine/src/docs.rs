use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ApiModuleDocs {
    pub name: String,
    pub methods: HashMap<String, MethodDoc>,
}

impl ApiModuleDocs {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            methods: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MethodDoc {
    pub name: String,
    pub description: String,
    pub params: Vec<ParamDoc>,
    pub returns: String,
}

impl MethodDoc {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            params: Vec::new(),
            returns: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParamDoc {
    pub name: String,
    pub type_name: String,
    pub description: String,
}

// Include the generated documentation
include!(concat!(env!("OUT_DIR"), "/api_docs.rs"));
