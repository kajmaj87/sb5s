-- help.lua - Documentation parser and help system
local DocSystem = {}

-- Constants for cleaner code
local SEPARATOR_WIDTH = 80
local TYPE_MODULE = "module"
local TYPE_FUNCTION = "function"

-- Module state
DocSystem.docsData = {}

-- Helper functions for parsing
local function trimString(str)
    return str:match("^%s*(.-)%s*$") or ""
end

local function isCommentLine(line)
    return line:match("^%-%-%-")
end

local function isNonTagComment(line)
    return isCommentLine(line) and not line:match("@")
end

local function getCommentContent(line)
    return trimString(line:match("^%-%-%-(.*)$") or "")
end

local function parseClassDeclaration(line)
    return line:match("%-%-%-@class%s+([%w%.]+)%s*(.*)$")
end

local function parseFunctionDeclaration(line)
    local full_name = line:match("function%s+([%w%.]+%.[%w_]+)%(")
    if full_name then
        return full_name:match("(.+)%.([^%.]+)$")
    end
    return nil, nil
end

local function parseParameterDeclaration(line)
    local name, type, desc = line:match("%-%-%-@param%s+(%S+)%s+([%w%|%.]+)%s*(.*)$")
    return name, type, desc
end

local function parseReturnDeclaration(line)
    local type, name, desc = line:match("%-%-%-@return%s+([%w%|%.]+)%s+(%S+)%s*(.*)$")
    return type, name, desc
end

-- Format helpers
local function separator(width)
    return string.rep("-", width or SEPARATOR_WIDTH)
end

local function formatParams(params)
    if not params or #params == 0 then
        return "()"
    end

    local parts = {}
    for _, param in ipairs(params) do
        table.insert(parts, param.name .. ": " .. param.type)
    end

    return "(" .. table.concat(parts, ", ") .. ")"
end

local function formatReturns(returns)
    if not returns or #returns == 0 then
        return ""
    end

    local parts = {}
    for _, ret in ipairs(returns) do
        local displayName = ret.name and ret.name ~= "" and ret.name or nil
        if displayName then
            table.insert(parts, displayName .. ": " .. ret.type)
        else
            table.insert(parts, ret.type)
        end
    end

    return " -> " .. table.concat(parts, ", ")
end

-- Parse documentation content
function DocSystem.parseContent(content)
    local docs = {}
    local function_comments = {}

    -- Split content into lines
    local lines = {}
    for line in content:gmatch("[^\r\n]+") do
        table.insert(lines, line)
    end

    for i, line in ipairs(lines) do
        -- Module definition
        local class_name, class_desc = parseClassDeclaration(line)
        if class_name then
            -- Create or update module entry
            docs[class_name] = docs[class_name] or {
                type = "table",
                description = class_desc or "",
                members = {}
            }

            -- Collect additional module description lines
            local j = i + 1
            while j <= #lines and isCommentLine(lines[j]) and not lines[j]:match("@") do
                local comment = getCommentContent(lines[j])
                if comment ~= "" then
                    if docs[class_name].description == "" then
                        docs[class_name].description = comment
                    else
                        docs[class_name].description = docs[class_name].description .. "\n" .. comment
                    end
                end
                j = j + 1
            end

            -- Function definition
        elseif line:match("function%s+[%w%.]+%.[%w_]+%(") then
            local module_name, func_name = parseFunctionDeclaration(line)

            if module_name and func_name then
                -- Ensure module exists
                docs[module_name] = docs[module_name] or {
                    type = "table",
                    description = "",
                    members = {}
                }

                -- Create function entry with collected description
                docs[module_name].members[func_name] = {
                    type = "function",
                    description = table.concat(function_comments, "\n"),
                    params = {},
                    returns = {}
                }

                local func_docs = docs[module_name].members[func_name]

                -- Process parameters and returns (looking backward)
                local j = i - 1
                while j >= 1 do
                    local prev = lines[j]

                    -- Stop at non-comment or class definition
                    if not isCommentLine(prev) or prev:match("@class") then
                        break
                    end

                    -- Parameter declaration
                    local param_name, param_type, param_desc = parseParameterDeclaration(prev)
                    if param_name then
                        table.insert(func_docs.params, 1, {
                            name = param_name,
                            type = param_type or "any",
                            description = param_desc or ""
                        })

                        -- Return declaration
                    else
                        local ret_type, ret_name, ret_desc = parseReturnDeclaration(prev)
                        if ret_type then
                            table.insert(func_docs.returns, 1, {
                                name = ret_name or "",
                                type = ret_type or "any",
                                description = ret_desc or ""
                            })
                        end
                    end

                    j = j - 1
                end

                -- Clear comments for next function
                function_comments = {}
            end

            -- Regular comment line (potential function description)
        elseif isNonTagComment(line) then
            local comment = getCommentContent(line)
            if comment ~= "" then
                table.insert(function_comments, comment)
            end
        end
    end

    return docs
end

-- Initialize documentation from file contents
function DocSystem.initDocs(file_contents)
    local loaded_files = {}
    DocSystem.docsData = {}

    for filename, content in pairs(file_contents) do
        table.insert(loaded_files, filename)
        local docs = DocSystem.parseContent(content)

        for k, v in pairs(docs) do
            DocSystem.docsData[k] = v
        end
    end

    return loaded_files
end

-- Split a path into components
local function splitPath(path)
    if not path then
        return {}
    end

    local parts = {}
    for part in path:gmatch("[^%.]+") do
        table.insert(parts, part)
    end
    return parts
end

-- Find an item by path
function DocSystem.findItem(path)
    if not path then
        return nil
    end

    -- Direct module lookup
    if DocSystem.docsData[path] then
        return DocSystem.docsData[path], TYPE_MODULE, path
    end

    local parts = splitPath(path)

    -- Two-part path: module.function
    if #parts == 2 then
        local module, func = parts[1], parts[2]

        if DocSystem.docsData[module] and
                DocSystem.docsData[module].members and
                DocSystem.docsData[module].members[func] then
            return DocSystem.docsData[module].members[func], TYPE_FUNCTION, module, func
        end
    end

    -- Three-part path: parent.module.function
    if #parts == 3 then
        local parent, module, func = parts[1], parts[2], parts[3]
        local compound_module = parent .. "." .. module

        if DocSystem.docsData[compound_module] and
                DocSystem.docsData[compound_module].members and
                DocSystem.docsData[compound_module].members[func] then
            return DocSystem.docsData[compound_module].members[func], TYPE_FUNCTION, compound_module, func
        end
    end

    return nil
end

-- Create a simple output formatter
local function createOutput()
    local output = {}

    local function append(str)
        table.insert(output, str or "")
    end

    local function get()
        return table.concat(output, "\n")
    end

    return { append = append, get = get }
end

-- Format module help output
function DocSystem.formatModuleHelp(module_info, module_name)
    local output = createOutput()

    output.append(separator())
    output.append(module_name)
    output.append(separator())
    output.append(module_info.description or "")
    output.append(separator())

    -- List members
    output.append("Members:")

    if module_info.members and next(module_info.members) then
        -- Sort member names for consistent output
        local member_names = {}
        for name in pairs(module_info.members) do
            table.insert(member_names, name)
        end
        table.sort(member_names)

        for _, name in ipairs(member_names) do
            local info = module_info.members[name]
            if info.type == "function" then
                local desc = info.description and info.description:match("^[^\n]+") or ""
                local desc_suffix = desc ~= "" and " - " .. desc or ""

                output.append("  " .. name .. formatParams(info.params) ..
                        formatReturns(info.returns) .. desc_suffix)
            else
                output.append("  " .. name .. " (" .. (info.type or "unknown") .. ")")
            end
        end
    else
        output.append("  No members found for this module.")
    end

    -- Help hint
    output.append("\nType help(\"" .. module_name .. ".member\") for details on a specific member.")

    return output.get()
end

-- Format function help output
function DocSystem.formatFunctionHelp(func_info, module_name, func_name)
    local output = createOutput()
    local full_name = module_name .. "." .. func_name

    output.append(separator())
    output.append(full_name)
    output.append(separator())
    output.append(func_info.description or "")
    output.append(separator())

    -- Function syntax
    output.append("Syntax: " .. func_name .. formatParams(func_info.params) ..
            formatReturns(func_info.returns))

    -- Parameters
    if func_info.params and #func_info.params > 0 then
        output.append("\nParameters:")
        for _, param in ipairs(func_info.params) do
            output.append("  " .. param.name .. " (" .. param.type .. "): " ..
                    (param.description or ""))
        end
    end

    -- Return values
    if func_info.returns and #func_info.returns > 0 then
        output.append("\nReturns:")
        for _, ret in ipairs(func_info.returns) do
            local name = ret.name ~= "" and ret.name or "result"
            output.append("  " .. name .. " (" .. ret.type .. "): " ..
                    (ret.description or ""))
        end
    end

    return output.get()
end

-- Format module list
function DocSystem.formatModuleList()
    local output = createOutput()

    output.append("Available modules:")
    output.append(separator())

    -- Get sorted module names
    local module_names = {}
    for name in pairs(DocSystem.docsData) do
        table.insert(module_names, name)
    end
    table.sort(module_names)

    -- Show modules with first line of description
    for _, name in ipairs(module_names) do
        local desc = DocSystem.docsData[name].description or ""
        local first_line = desc:match("^[^\n]+") or desc
        output.append(name .. " - " .. first_line)
    end

    output.append("\nType help(\"module\") or help(\"module.function\") for details.")

    return output.get()
end

-- Get help text for a path
function DocSystem.getHelpText(path)
    -- Check if docs are loaded
    if not DocSystem.docsData or next(DocSystem.docsData) == nil then
        return "No documentation has been loaded. Please make sure initDocs() was called."
    end

    -- List all modules if no path provided
    if not path then
        return DocSystem.formatModuleList()
    end

    -- Find the requested item
    local item, item_type, module_name, func_name = DocSystem.findItem(path)

    -- Handle not found
    if not item then
        local first_part = path:match("^([^%.]+)")
        if DocSystem.docsData[first_part] then
            return "Could not find: " .. path
        else
            return "Could not find module: " .. first_part
        end
    end

    -- Format output based on item type
    if item_type == TYPE_FUNCTION then
        return DocSystem.formatFunctionHelp(item, module_name, func_name)
    else
        return DocSystem.formatModuleHelp(item, module_name)
    end
end

-- The main help function
function DocSystem.help(path)
    local helpText = DocSystem.getHelpText(path)
    print(helpText)
    return helpText
end

-- Install global functions
function DocSystem.installGlobals()
    _G.init_help_docs = function(file_contents)
        local loaded = DocSystem.initDocs(file_contents)
        local count = #loaded

        if count > 0 then
            print("Documentation helper loaded from " .. count ..
                    " files. Type help() to see available modules.")
        else
            print("Warning: No documentation files were loaded.")
        end

        return true
    end

    _G.help = function(path)
        return DocSystem.help(path)
    end

    return true
end

-- Set up globals
DocSystem.installGlobals()

return DocSystem
