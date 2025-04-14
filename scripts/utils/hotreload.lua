STATE = {}

--- Manages persistent state across hot reloads.
---
--- This function allows saving and retrieving state associated with a specific key.
--- When a handler function is provided, it is executed only once to initialize the state,
--- and its result is stored. On subsequent calls, even after a hot reload, the stored
--- state is returned without re-executing the handler.
--- If the function is not provided, the state is simply retrieved.
---
--- @param key any The key used to identify the state.
--- @param handler function|nil An optional function that initializes the state. It is called only once. Its value is stored and returned in subsequent calls.
--- @return table The stored state associated with the given key.
function state(key, handler)
    if not STATE[key] and handler then
        STATE[key] = {}
        if handler then
            STATE[key] = handler()
        end
    end
    return STATE[key]
end