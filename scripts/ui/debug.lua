local fps_history = {}
local fps_history_size = 1000

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

function debug_window.draw()
    ui.label(20, 230, function()
        return string.format("FPS: %d (Avg: %.1f)", ui.fps(), get_avg_fps())
    end)
    ui.label(20, 260, function()
        local x, y = ui.tile.hovered()
        local id = ui.tile.at(x, y)
        if not id then
            return string.format("Hover: (%d, %d) [Empty]", x, y)
        else
            return string.format("Hover: (%d, %d) ID: %d", x, y, id)
        end
    end)
end

return debug_window
