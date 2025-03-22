local window_open = false
button("Click Me", function()
    if window_open then
        return
    end
    window("Test Window", function()
        window_open = true
        plot("Sine Wave", function()
            local data = {}
            for i = 1, 100 do
                data[i] = math.sin(i * 0.1)
            end
            return data
        end)
    end)
end)
lua_console("help()")
local help_old = help;
help = function(a)
    print(help_old(a))
end

label(function()
    return "Hello World"
end)