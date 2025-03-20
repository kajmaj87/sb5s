use crate::lua_engine::LuaCommand;
use std::sync::mpsc;

pub struct LuaClient {
    command_tx: mpsc::Sender<LuaCommand>,
}

impl LuaClient {
    pub fn new(command_tx: mpsc::Sender<LuaCommand>) -> Self {
        Self { command_tx }
    }

    pub fn run_script(&self, script: &str) -> Result<(), String> {
        let (response_tx, response_rx) = mpsc::channel();
        self.command_tx
            .send(LuaCommand::RunScript {
                script: script.to_string(),
                response_tx,
            })
            .unwrap();
        response_rx.recv().unwrap()
    }

    pub fn execute(&self, code: &str) -> Result<String, String> {
        let (response_tx, response_rx) = mpsc::channel();
        self.command_tx
            .send(LuaCommand::Execute {
                code: code.to_string(),
                response_tx,
            })
            .unwrap();
        response_rx.recv().unwrap()
    }
    pub fn execute_non_blocking(&self, code: &str) -> mpsc::Receiver<Result<String, String>> {
        let (response_tx, response_rx) = mpsc::channel();

        self.command_tx
            .send(LuaCommand::Execute {
                code: code.to_string(),
                response_tx,
            })
            .unwrap();

        // Return the receiver immediately without waiting
        response_rx
    }

    // Register a callback - uses a local Lua instance temporarily for conversion
    pub fn register_callback(&self, script: String) -> u32 {
        // Create registry key to share across thread boundary
        let (response_tx, response_rx) = mpsc::channel();
        self.command_tx
            .send(LuaCommand::RegisterCallback {
                script,
                response_tx,
            })
            .unwrap();
        response_rx.recv().unwrap()
    }

    // Other client methods as needed...
}
