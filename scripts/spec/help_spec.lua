-- spec/help_spec.lua

describe("Documentation Help System", function()
    local DocSystem

    -- Mock file contents for testing
    local test_files = {
        ["test_basic.lua"] = [[
---@class test.basic
--- Basic test module
test.basic = {}

---@param x number The x value
---@param y number The y value
---@return number sum The sum of x and y
--- Add two numbers together
function test.basic.add(x, y)
    return x + y
end

---@param x number The first number
---@return number result The square of the number
--- Calculate the square of a number
function test.basic.square(x)
    return x * x
end
]],
        ["test_nested.lua"] = [[
---@class test.nested
--- Nested module for testing hierarchies
test.nested = {}

---@class test.nested.submodule
--- A submodule within the nested module
test.nested.submodule = {}

---@param value string The string to process
---@return string result The processed string
--- Process a string value
function test.nested.submodule.process(value)
    return value
end
]],
        ["test_fields.lua"] = [[
---@class test.fields
--- Module with field definitions
test.fields = {}

---@field max_size number The maximum size allowed
---@field name string The name of the field container
test.fields.config = {
    max_size = 100,
    name = "Default"
}

---@param size number The size to validate
---@return boolean valid Whether the size is valid
--- Check if a size is within limits
function test.fields.validate(size)
    return size <= test.fields.config.max_size
end
]],
        ["test_complex.lua"] = [[
---@class test.complex
--- A more complex module demonstrating multiple features
--- This description spans
--- multiple lines
test.complex = {}

---@field DEFAULT_TIMEOUT number Default timeout in milliseconds
test.complex.DEFAULT_TIMEOUT = 1000

---@param callback function The callback to execute
---@param timeout number|nil Optional timeout override
---@return boolean success Whether the operation succeeded
---@return string|nil error Error message if failed
--- Executes a callback with timeout
--- Returns success status and error if applicable
function test.complex.execute(callback, timeout)
    -- Implementation would go here
    return true, nil
end
]]
    }

    setup(function()
        -- Load the DocSystem module
        package.loaded.DocSystem = nil  -- Ensure we get a fresh load
        DocSystem = require("help")

        -- Initialize with test files
        DocSystem.initDocs(test_files)
    end)

    describe("help function", function()
        _G.print = function()
        end

        it("should list all modules when called without arguments", function()
            local result = DocSystem.help()

            assert.matches("test.basic", result)
            assert.matches("test.nested", result)
            assert.matches("test.fields", result)
            assert.matches("test.complex", result)
        end)

        it("should display module information", function()
            local result = DocSystem.help("test.basic")

            assert.matches("Basic test module", result)
            assert.matches("add", result)
            assert.matches("square", result)
        end)

        it("should display nested module information", function()
            local result = DocSystem.help("test.nested.submodule")

            assert.matches("A submodule within", result)
            assert.matches("process", result)
        end)

        it("should display function information", function()
            local result = DocSystem.help("test.basic.add")

            assert.matches("Add two numbers together", result)
            assert.matches("Parameters:", result)
            assert.matches("x %(number%)", result)
            assert.matches("y %(number%)", result)
            assert.matches("Returns:", result)
            assert.matches("sum %(number%)", result)
        end)

        it("should handle complex function information", function()
            local result = DocSystem.help("test.complex.execute")

            assert.matches("Executes a callback with timeout", result)
            assert.matches("callback %(function%)", result)
            assert.matches("timeout %(number|nil%)", result)
            assert.matches("success %(boolean%)", result)
            assert.matches("error %(string|nil%)", result)
        end)

        it("should handle invalid paths", function()
            local result = DocSystem.help("nonexistent.module")

            assert.matches("Could not find module", result)
        end)

        it("should handle empty documentation", function()
            -- Temporarily replace docs with empty data
            local originalDocs = DocSystem.docsData
            DocSystem.docsData = {}

            local result = DocSystem.help()
            assert.matches("No documentation has been loaded", result)

            -- Restore original docs
            DocSystem.docsData = originalDocs
        end)
    end)

    -- Test global compatibility
    describe("global help function", function()
        it("should make help function available globally", function()
            assert.is_function(_G.help)

            local result = _G.help("test.basic")
            assert.matches("Basic test module", result)
        end)

        it("should make init_help_docs function available globally", function()
            assert.is_function(_G.init_help_docs)

            -- Test with a simple file
            local simple_files = {
                ["simple.lua"] = [[
---@class simple.test
--- A simple test module
simple.test = {}
]]
            }

            _G.init_help_docs(simple_files)
            local result = _G.help("simple.test")
            assert.matches("A simple test module", result)
        end)
    end)
end)
