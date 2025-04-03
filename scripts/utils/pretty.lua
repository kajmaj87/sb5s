-- Store the original print function
local original_print = print

-- Table to string conversion function
local function table_to_string(t, options, printed)
    -- Handle non-table cases
    if type(t) ~= "table" then
        return tostring(t)
    end

    -- Default options
    options = options or {}
    local indent_str = options.indent or "  "
    local depth = options.depth or 0
    local max_depth = options.max_depth or 10
    local show_address = options.show_address ~= false -- default true
    local sort_keys = options.sort_keys ~= false -- default true
    local newline = options.newline or "\n"

    -- Track already printed tables to handle cycles
    printed = printed or {}
    if printed[t] then
        return show_address and string.format("<table: %p, cyclic reference>", t) or "<cyclic reference>"
    end
    printed[t] = true

    -- Create indentation string based on depth
    local indent = string.rep(indent_str, depth)
    local indent_next = string.rep(indent_str, depth + 1)

    -- Start building the result
    local result = {}

    -- Check if the table is empty
    if next(t) == nil then
        return "{}"
    end

    -- Check if we're at max depth
    if depth >= max_depth then
        return show_address and string.format("<table: %p, max depth reached>", t) or "<max depth reached>"
    end

    -- Check if it's an array (consecutive numeric indices)
    local is_array = true
    local max_index = 0
    for k, _ in pairs(t) do
        if type(k) ~= "number" or k <= 0 or math.floor(k) ~= k then
            is_array = false
            break
        end
        max_index = math.max(max_index, k)
    end

    -- Count actual elements to check if it's a sparse array
    if is_array and max_index > 0 then
        local count = 0
        for _ in pairs(t) do
            count = count + 1
        end
        is_array = (count == max_index)
    end

    -- Open table
    table.insert(result, "{" .. newline)

    -- Get all keys and sort them
    local keys = {}
    for k in pairs(t) do
        table.insert(keys, k)
    end

    if sort_keys then
        table.sort(keys, function(a, b)
            -- If it's an array, sort numerically
            if is_array then
                return a < b
            end
            -- Sort by type first
            local type_a, type_b = type(a), type(b)
            if type_a ~= type_b then
                return type_a < type_b
            end
            -- Then by value
            if type_a == "number" or type_a == "string" then
                return a < b
            end
            -- For other types (function, userdata, etc.), convert to string
            return tostring(a) < tostring(b)
        end)
    end

    -- Process each key-value pair
    for i, k in ipairs(keys) do
        local v = t[k]
        local key_str

        if is_array then
            key_str = ""  -- For arrays, don't show the index
        elseif type(k) == "string" and k:match("^[%a_][%a%d_]*$") then
            -- Simple identifier keys don't need quotes
            key_str = k .. " = "
        elseif type(k) == "string" then
            -- String keys with special characters need quotes
            key_str = string.format("[%q] = ", k)
        else
            -- Other types of keys (numbers, booleans, etc.)
            key_str = string.format("[%s] = ", tostring(k))
        end

        -- Format value based on its type
        local val_str
        if type(v) == "table" then
            -- Recursively convert nested tables
            local nested_options = {
                indent = indent_str,
                depth = depth + 1,
                max_depth = max_depth,
                show_address = show_address,
                sort_keys = sort_keys,
                newline = newline
            }
            val_str = table_to_string(v, nested_options, printed)
        elseif type(v) == "string" then
            val_str = string.format("%q", v)
        else
            val_str = tostring(v)
        end

        -- Add the key-value pair to result
        table.insert(result, string.format("%s%s%s%s",
                indent_next, key_str, val_str, i < #keys and "," or ""))
        table.insert(result, newline)
    end

    -- Close table
    table.insert(result, indent .. "}")

    -- Remove reference to avoid affecting other calls
    printed[t] = nil

    return table.concat(result, "")
end

-- Replace the global print function with our enhanced version
print = function(...)
    local args = { ... }
    local n = select('#', ...)

    if n == 0 then
        -- Print a blank line if no arguments
        return original_print()
    end

    -- Convert args to strings, except tables which get pretty-printed
    local formatted = {}
    for i = 1, n do
        if type(args[i]) == "table" then
            formatted[i] = table_to_string(args[i])
        else
            formatted[i] = tostring(args[i])
        end
    end

    -- Call original print with formatted arguments
    return original_print(unpack(formatted, 1, n))
end

---@function print_compact
---
--- Prints values to the console in a compact one-line format. Tables are serialized
--- without indentation or newlines, making them more concise for logging and debugging.
--- This is especially useful for displaying data structures in a space-efficient manner.
---
--- Similar to the standard print function, it can take any number of arguments of any type.
--- Tables are automatically serialized into a condensed format, while non-table values
--- are converted to strings using tostring().
---
---@param ... any Arguments to print (any number of values of any type)
---@return nil
function print_compact(...)
    local args = { ... }
    local n = select('#', ...)

    if n == 0 then
        return original_print()
    end

    local formatted = {}
    local compact_options = { newline = "", indent = "" }

    for i = 1, n do
        if type(args[i]) == "table" then
            formatted[i] = table_to_string(args[i], compact_options)
        else
            formatted[i] = tostring(args[i])
        end
    end

    return original_print(unpack(formatted, 1, n))
end