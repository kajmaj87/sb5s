local fps_history = {}
local fps_history_size = 250

local function update()
    local fps = ui.fps()
    table.insert(fps_history, fps)
    -- Trim history if needed
    while #fps_history > fps_history_size do
        table.remove(fps_history, 1)
    end
end

local function get_avg_fps()
    update()
    local sum = 0
    for _, fps in ipairs(fps_history) do
        sum = sum + fps
    end
    return sum / #fps_history
end

local debug_window = {}
local mouse_state

ui.input.register_mouse_move(function(x, y)
    mouse_state = string.format("Mouse at: %d, %d", x, y)
end)

ui.input.register_mouse_wheel(function(dx)
    mouse_state = string.format("Mouse scrolled by: %d", dx)
end)

ui.input.register_mouse_wheel(function(dx)
    mouse_state = string.format("Mouse scrolled by: %d", dx)
end)

local events = 0
api.projection.register("PeopleCounter", function(event)
    events = events + 1
end)

function debug_window.draw()
    ui.label(20, 270, function()
        return string.format("FPS: %d (Avg: %.0f)", ui.fps(), get_avg_fps())
    end)
    ui.label(20, 300, function()
        local x, y = ui.tile.hovered()
        local id = ui.tile.at(x, y)
        if id then
            return string.format("Hover: (%d, %d) ID: %d", x, y, id)
        else
            return string.format("Hover: (%d, %d) [Empty]", x, y)
        end
    end)
    ui.label(20, 330, function()
        local x, y, count = api.location.most_crowded()
        if count and count > 0 then
            return string.format("Most crowded: (%d, %d) - %d people", x, y, count)
        else
            return "No people on map yet"
        end
    end)
    ui.label(20, 360, function()
        if mouse_state then
            return mouse_state
        else
            return "Do something with the mouse!"
        end
    end)
    ui.label(20, 390, function()
        return string.format("Events received: %d", events)
    end)
end
debug_window:draw()
return debug_window
