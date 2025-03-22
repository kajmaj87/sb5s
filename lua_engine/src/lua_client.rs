use crate::lua_engine::LuaCommand;
use std::sync::mpsc;

pub struct LuaClient {
    command_tx: mpsc::Sender<LuaCommand>,
}

impl LuaClient {
    pub fn new(command_tx: mpsc::Sender<LuaCommand>) -> Self {
        Self { command_tx }
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
}
