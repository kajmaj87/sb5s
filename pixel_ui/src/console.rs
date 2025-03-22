use arboard::Clipboard;
use lua_engine::lua_client::LuaClient;
use macroquad::hash;
use macroquad::prelude::*;
use macroquad::ui::{root_ui, widgets};
use std::sync::{mpsc, Arc};

pub struct Console {
    pub(crate) visible: bool,
    history: Vec<String>,
    editbox: String,
    clipboard: Option<Clipboard>,
    lua_client: Arc<LuaClient>,
    pending_commands: Vec<mpsc::Receiver<Result<String, String>>>,
}

impl Console {
    pub(crate) fn new(lua_client: Arc<LuaClient>) -> Self {
        // Initialize clipboard
        let clipboard = match Clipboard::new() {
            Ok(clipboard) => Some(clipboard),
            Err(e) => {
                println!("Failed to initialize clipboard: {:?}", e);
                None
            }
        };

        Self {
            visible: false,
            history: vec![
                "Welcome to the console! Type help() to start exploring the api.".to_string(),
            ],
            editbox: String::new(),
            clipboard,
            lua_client,
            pending_commands: Default::default(),
        }
    }

    fn execute_command(&mut self) {
        let command = self.editbox.clone();
        if command.is_empty() {
            return;
        }

        // Add user input to history
        self.history.push(format!("> {}", command));

        // Execute the script with LuaEngine
        let pending_result = self.lua_client.execute_non_blocking(command.as_str());
        self.pending_commands.push(pending_result);
    }

    pub(crate) fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub(crate) fn update(&mut self) {
        // Check all pending command results without blocking
        let mut completed = Vec::new();

        for (i, receiver) in self.pending_commands.iter().enumerate() {
            match receiver.try_recv() {
                Ok(result) => {
                    // Process the result
                    match result {
                        Ok(output) => self.history.push(output),
                        Err(err) => self.history.push(format!("Error: {}", err)),
                    }
                    // Mark this receiver as completed
                    completed.push(i);
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // Not ready yet, continue with other tasks
                    continue;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    // Sender was dropped without sending
                    self.history.push("Command processing failed".to_string());
                    completed.push(i);
                }
            }
        }
        // Remove completed receivers (in reverse order to avoid index issues)
        for i in completed.into_iter().rev() {
            self.pending_commands.remove(i);
        }
        // Limit history size
        while self.history.len() > 100 {
            self.history.remove(0);
        }

        if !self.visible {
            return;
        }

        // Handle clipboard operations
        let copy_requested = is_key_down(KeyCode::LeftControl) && is_key_pressed(KeyCode::C)
            || (is_key_down(KeyCode::LeftControl) && is_key_pressed(KeyCode::Insert));
        if copy_requested {
            if let Some(ref mut ctx) = self.clipboard {
                let _ = ctx.set_text(self.editbox.clone());
                self.history.push("Text copied to clipboard".to_string());
            }
        }

        let paste_requested = (is_key_down(KeyCode::LeftControl) && is_key_pressed(KeyCode::V))
            || (is_key_down(KeyCode::LeftShift) && is_key_pressed(KeyCode::Insert))
            || (is_key_down(KeyCode::RightShift) && is_key_pressed(KeyCode::Insert));
        // Paste (Ctrl+V)
        if paste_requested {
            if let Some(ref mut ctx) = self.clipboard {
                if let Ok(clipboard_text) = ctx.get_text() {
                    self.editbox.push_str(&clipboard_text);
                }
            }
        }

        // Execute command on Shift+Enter
        if is_key_pressed(KeyCode::Enter)
            && (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl))
        {
            self.execute_command();
        }
    }

    pub(crate) fn draw(&mut self) {
        if !self.visible {
            return;
        }

        // Calculate console dimensions
        let console_height = screen_height() * 0.4;
        let input_area_height = 180.0;

        // Draw semi-transparent background
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            console_height,
            Color::new(0.1, 0.1, 0.1, 0.7), // TEXT_BACKGROUND_COLOR
        );

        // Draw input area with slightly darker background
        draw_rectangle(
            0.0,
            console_height - input_area_height,
            screen_width(),
            input_area_height,
            Color::new(0.0, 0.0, 0.0, 0.8),
        );

        // Draw command prompt
        draw_text(
            "> ",
            10.0,
            console_height - input_area_height + 25.0,
            20.0,
            WHITE,
        );

        // Draw command history (most recent at the bottom)
        let line_height = 20.0;
        let visible_lines = ((console_height - input_area_height) / line_height) as usize;
        let start_idx = if self.history.len() > visible_lines {
            self.history.len() - visible_lines
        } else {
            0
        };
        for (i, line) in self.history[start_idx..].iter().enumerate() {
            let y = (i as f32) * line_height + 20.0;
            draw_text(line, 10.0, y, 20.0, WHITE);
        }

        // Use Editbox for input (placed after background drawing)
        let mut ui = root_ui();

        // Position the editbox in the input area - using Vec2 for size
        let editbox_width = screen_width() - 40.0;

        // Create editbox with proper size (Vec2)
        let size = Vec2::new(editbox_width, input_area_height);
        let pos_x = 35.0; // After the prompt
        let pos_y = console_height - input_area_height + 5.0;

        widgets::Editbox::new(hash!(), size)
            .position(Vec2::new(pos_x, pos_y))
            .multiline(true)
            .ui(&mut ui, &mut self.editbox);

        ui.pop_skin();
    }
}
