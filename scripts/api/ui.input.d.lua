---@class ui.input Input handling functionality
ui.input = {}

---Register a keyboard shortcut handler
---@param key_combo string Key combination string (e.g., "X", "SHIFT+X", "CTRL+S", "ESCAPE")
---@param handler function Function to call when the shortcut is triggered
---@return nil
function ui.input.register_shortcut(key_combo, handler)
end

---Register a mouse button handler
---@param button string Mouse button identifier ("LMB" for left, "RMB" for right, "MMB" for middle)
---@param handler function Function to call when the button is pressed, receives (x, y) screen coordinates
---@return nil
function ui.input.register_mouse(button, handler)
end

---Register a mouse drag handler
---@param button string Mouse button identifier for the drag ("LMB", "RMB", or "MMB")
---@param handler function Function to call during drag, receives (deltaX, deltaY) movement values
---@return nil
function ui.input.register_drag(button, handler)
end

---Register a mouse movement handler
---@param handler function Function to call when the mouse moves, receives (x, y) screen coordinates
---@return nil
function ui.input.register_mouse_move(handler)
end

---Register a mouse wheel handler
---@param handler function Function to call when the mouse wheel is scrolled, receives delta value
---@return nil
function ui.input.register_mouse_wheel(handler)
end

---Unregister an event handler
---@param event_id string Event identifier to unregister (e.g., "LMB", "SHIFT+X", "LMB_DRAG")
---@return boolean success Whether the event was successfully unregistered
function ui.input.unregister(event_id)
end