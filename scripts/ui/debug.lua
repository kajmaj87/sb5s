local fps_history = {}
local fps_history_size = 1000

local function update()
    local fps = ui.get_fps()
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

function debug_window.draw()
    ui.label(20, 230, function()
        return string.format("FPS: %d (Avg: %.1f)", ui.get_fps(), get_avg_fps())
    end)
end

return debug_window
