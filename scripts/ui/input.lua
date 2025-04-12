-- Register a keyboard shortcut
local origin = ui.input.register_shortcut
local shortcuts = {}
ui.input.register_shortcut = function(combo, description, handler)
    shortcuts[combo] = { description = description }
    origin(combo, function()
        print("Shortcut triggered:", combo, description)
        handler()
    end)
end

ui.input.register_shortcut("ESCAPE", "Exit mode", function()
end)
ui.input.register_shortcut("R", "Print R", function()
end)
ui.input.register_shortcut("F1", "Print keyboard shortcuts", function()
    for combo, data in pairs(shortcuts) do
        print("Shortcut:", combo, "Description:", data.description)
    end
end)
-- Register a mouse click handler
ui.input.register_mouse("LMB", function()
    local x, y = ui.tile.hovered()
    local person = api.person.create("testa", x, y)
    print_compact("Person:", person)
end)
