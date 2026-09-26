--------------------------------------------------------------------------------
-- Game engine emulation for the Lua test harness.
--
-- This replaces the C++ side of the game's scripting environment, so the real script libraries of the game and
-- the scripts of mods can run outside of the game. It's loaded before anything else, receiving from RPFM:
--   * rpfm_read_script(path): returns the code of a script, or nil if it doesn't exist.
--   * rpfm_list_scripts(folder, filter): returns a comma-separated list of the scripts in a folder.
--   * rpfm_returns: return type of each documented method, by owner, then by method.
--   * rpfm_accessors: return type of each documented context accessor, by event, then by accessor.
--   * rpfm_faction_keys, rpfm_region_keys: sets of the keys of the factions and regions in the game's DB.
--
-- Everything else talks to the game through objects created here. Each call on them is recorded, and returns
-- either what the test configured, or a default value of the type the docs say the method returns.
--------------------------------------------------------------------------------

local read_script = rpfm_read_script
local list_scripts = rpfm_list_scripts
local api_returns = rpfm_returns
local api_accessors = rpfm_accessors
local faction_keys = rpfm_faction_keys
local region_keys = rpfm_region_keys
rpfm_read_script, rpfm_list_scripts, rpfm_returns, rpfm_accessors, rpfm_faction_keys, rpfm_region_keys = nil, nil, nil, nil, nil, nil

rpfm = {
    -- Every call done on engine objects, in order.
    calls = {},

    -- Errors raised by listeners, mods and loaders.
    errors = {},

    -- Methods called on engine objects that the docs don't describe, as "type:method".
    unmocked = {},

    -- Output of the scripts.
    log = {},

    -- Registered tests.
    tests = {},

    -- Contents of the files written by the scripts, by path.
    files = {},

    -- Game state the engine objects expose.
    world = {
        turn = 1,
        factions = {},
        faction_order = {},
        regions = {},
        region_order = {},
    },
};

local MAX_LOG_LINES = 5000;
local unpack = unpack;
local original_find = string.find;
local original_len = string.len;
local original_sub = string.sub;

local function log(line)
    if #rpfm.log < MAX_LOG_LINES then
        table.insert(rpfm.log, tostring(line));
    end;
end;

local function pack(...)
    return { n = select("#", ...), ... };
end;

--------------------------------------------------------------------------------
-- Sandbox. Scripts come from packs downloaded from anywhere, so they get no access to the system.
--------------------------------------------------------------------------------

os.execute, os.exit, os.remove, os.rename, os.tmpname, os.getenv = nil, nil, nil, nil, nil, nil;
package.loadlib = nil;
package.cpath = "";
debug.debug = nil;

-- Files are kept in memory. Files can only be read after being written by a script.
local function open_file(path, mode)
    path = tostring(path);
    mode = mode or "r";
    local writing = original_find(mode, "[wa]") ~= nil;
    if not writing and rpfm.files[path] == nil then
        return nil, path .. ": No such file or directory";
    end;

    if original_find(mode, "w") then
        rpfm.files[path] = "";
    end;

    local file = { position = 1 };
    function file:write(...)
        for i = 1, select("#", ...) do
            rpfm.files[path] = (rpfm.files[path] or "") .. tostring((select(i, ...)));
        end;
        return self;
    end;
    function file:read(format)
        local contents = rpfm.files[path] or "";
        if self.position > original_len(contents) then
            return nil;
        end;
        if format == "*a" or format == "*all" then
            local rest = original_sub(contents, self.position);
            self.position = original_len(contents) + 1;
            return rest;
        end;
        local line_end = original_find(contents, "\n", self.position, true) or (original_len(contents) + 1);
        local line = original_sub(contents, self.position, line_end - 1);
        self.position = line_end + 1;
        return line;
    end;
    function file:lines()
        return function() return self:read("*l") end;
    end;
    function file:flush() return self end;
    function file:close() return true end;
    function file:seek() return 0 end;
    function file:setvbuf() return true end;
    return file;
end;

io = { open = open_file, write = function(...) log(table.concat(pack(...), "")) end };

--------------------------------------------------------------------------------
-- Script loading, over the scripts of the game and the tested packs.
--------------------------------------------------------------------------------

-- Scripts refer to their files relative to the game's folder, with or without the data folder in front.
local function normalize_path(path)
    path = string.gsub(path, "\\", "/");
    path = string.gsub(path, "^%./", "");
    path = string.gsub(path, "^/", "");
    path = string.gsub(path, "^data/", "");
    return path;
end;

local function load_script(path)
    path = normalize_path(path);
    local source = read_script(path);
    if source == nil then
        return nil, "cannot open " .. path;
    end;

    local chunk, err = loadstring(source, "@" .. path);
    if chunk == nil then
        table.insert(rpfm.errors, err);
    end;
    return chunk, err;
end;

-- Finds a module or file name in the folders of package.path, like the game does.
local function find_script(name)
    local direct = normalize_path(name);
    if read_script(direct) ~= nil then
        return direct;
    end;

    local module_path = string.gsub(name, "%.", "/");
    for template in string.gmatch(package.path, "[^;]+") do
        local candidate = normalize_path((string.gsub(template, "%?", module_path)));
        if read_script(candidate) ~= nil then
            return candidate;
        end;
    end;

    return nil;
end;

package.loaders = {
    package.loaders[1],
    function(name)
        local path = find_script(name);
        if path == nil then
            return "\n\tno script '" .. name .. "' in the game or the tested packs";
        end;

        local chunk, err = load_script(path);
        return chunk or err;
    end,
};

function loadfile(name)
    local path = find_script(name);
    if path == nil then
        return nil, "cannot open " .. tostring(name);
    end;
    return load_script(path);
end;

function dofile(name)
    local chunk, err = loadfile(name);
    if chunk == nil then
        error(err, 2);
    end;
    return chunk();
end;

package.path = "script/?.lua;script/_lib/?.lua;?.lua";

--------------------------------------------------------------------------------
-- String functions, replaced by the game with versions that count UTF-8 characters instead of bytes.
--------------------------------------------------------------------------------

string.find_lua = original_find;
string.len_lua = original_len;
string.sub_lua = original_sub;

local function is_ascii(text)
    return original_find(text, "[\128-\255]") == nil;
end;

-- Byte position where each character starts, plus one past the end.
local function character_starts(text)
    local starts = {};
    local position = 1;
    local length = original_len(text);
    while position <= length do
        table.insert(starts, position);
        local byte = string.byte(text, position);
        if byte >= 240 then position = position + 4
        elseif byte >= 224 then position = position + 3
        elseif byte >= 192 then position = position + 2
        else position = position + 1 end;
    end;
    table.insert(starts, length + 1);
    return starts;
end;

function string.len(text)
    if is_ascii(text) then
        return original_len(text);
    end;
    return #character_starts(text) - 1;
end;

function string.sub(text, first, last)
    if is_ascii(text) then
        return original_sub(text, first, last);
    end;

    local starts = character_starts(text);
    local count = #starts - 1;
    first = first or 1;
    last = last or -1;
    if first < 0 then first = math.max(count + first + 1, 1) end;
    if last < 0 then last = count + last + 1 end;
    if first < 1 then first = 1 end;
    if last > count then last = count end;
    if first > last then return "" end;
    return original_sub(text, starts[first], starts[last + 1] - 1);
end;

-- Plain search, without patterns, returning character positions.
function string.find(text, substring, init)
    if is_ascii(text) then
        return original_find(text, substring, init, true);
    end;

    local starts = character_starts(text);
    local count = #starts - 1;
    init = init or 1;
    if init < 0 then init = math.max(count + init + 1, 1) end;
    if init > count + 1 then return nil end;

    local first_byte, last_byte = original_find(text, substring, starts[init], true);
    if first_byte == nil then
        return nil;
    end;

    local first, last;
    for index, start in ipairs(starts) do
        if start == first_byte then first = index end;
        if start == last_byte + 1 then last = index - 1 break end;
    end;
    return first, last;
end;

--------------------------------------------------------------------------------
-- Engine objects.
--------------------------------------------------------------------------------

local objects = setmetatable({}, { __mode = "k" });

local function is_object(value)
    return objects[value] ~= nil;
end;

local call_method;

local LIST_METHODS = { num_items = true, is_empty = true, item_at = true };

-- Types without docs accept any method, reported as unmocked. Documented types only have their documented methods,
-- so looking up anything else, like fields scripts check on objects, gets nil, as in the game.
local function has_method(data, key)
    if data.methods[key] ~= nil or key == "is_null_interface" or (data.items and LIST_METHODS[key]) then
        return true;
    end;
    if data.returns == nil or next(data.returns) == nil then
        return true;
    end;
    return data.returns[key] ~= nil;
end;

local function new_data(type_name, methods, fields, returns)
    return {
        type = type_name,
        methods = methods or {},
        fields = fields or {},
        returns = returns or api_returns[type_name],
    };
end;

-- Creates an engine object. Methods are the values its methods return: plain values, or functions called with
-- the arguments of the call. Fields are values read as `object.field` instead of called.
function rpfm.object(type_name, methods, fields, returns)
    local proxy = newproxy(true);
    local metatable = getmetatable(proxy);
    local data = new_data(type_name, methods, fields, returns);
    objects[proxy] = data;

    metatable.__index = function(_, key)
        if data.fields[key] ~= nil then
            return data.fields[key];
        end;
        if not has_method(data, key) then
            return nil;
        end;
        return function(_, ...) return call_method(data, key, ...) end;
    end;

    -- The game identifies objects by the start of their string, like "FACTION_SCRIPT_INTERFACE".
    metatable.__tostring = function()
        local label = data.methods.name;
        if type(label) ~= "string" then label = "" end;
        return type_name .. " (" .. label .. ")";
    end;

    return proxy;
end;

-- Creates a table of engine functions called with a dot, like `common.function()`, so there's no `self` to skip.
function rpfm.library(type_name, methods)
    local data = new_data(type_name, methods);
    local library = setmetatable({}, {
        __index = function(_, key)
            return function(...) return call_method(data, key, ...) end;
        end,
    });
    objects[library] = data;
    return library;
end;

function rpfm.null(type_name)
    local null = rpfm.object(type_name);
    objects[null].null = true;
    return null;
end;

-- Wraps objects in a list interface, like REGION_LIST_SCRIPT_INTERFACE.
function rpfm.list(type_name, items)
    local list = rpfm.object(type_name);
    objects[list].items = items or {};
    return list;
end;

local function default_value(type_name)
    if type_name == nil or type_name == "any" then return rpfm.object("UNKNOWN") end;
    if type_name == "nil" or type_name == "function" then return nil end;
    if type_name == "boolean" then return false end;
    if type_name == "number" then return 0 end;
    if type_name == "string" then return "" end;
    if type_name == "table" then return {} end;
    return rpfm.null(type_name);
end;

call_method = function(data, method, ...)
    local args = pack(...);
    table.insert(rpfm.calls, { type = data.type, method = method, args = args });

    local configured = data.methods[method];
    if configured ~= nil then
        if type(configured) == "function" then
            return configured(unpack(args, 1, args.n));
        end;

        -- Lists can be configured as plain arrays, and get wrapped in the list interface the docs say.
        local return_type = data.returns and data.returns[method];
        if type(configured) == "table" and not is_object(configured) and return_type and original_find(return_type, "_LIST_SCRIPT_INTERFACE$") then
            configured = rpfm.list(return_type, configured);
            data.methods[method] = configured;
        end;
        return configured;
    end;

    if method == "is_null_interface" then
        return data.null == true;
    end;

    if data.items then
        if method == "num_items" then return #data.items end;
        if method == "is_empty" then return #data.items == 0 end;
        if method == "item_at" then
            local item = data.items[(args[1] or 0) + 1];
            if item ~= nil then return item end;
            return default_value(data.returns and data.returns.item_at);
        end;
    end;

    local return_type = data.returns and data.returns[method];
    if return_type == nil then
        rpfm.unmocked[data.type .. ":" .. tostring(method)] = true;
    end;
    return default_value(return_type);
end;

--------------------------------------------------------------------------------
-- World fixture.
--------------------------------------------------------------------------------

local function ordered(keys, by_key)
    local items = {};
    for _, key in ipairs(keys) do
        table.insert(items, by_key[key]);
    end;
    return items;
end;

-- Factions and regions of the game's DB exist even when tests don't set them up, like in the game.
local function faction_by_key(key)
    if rpfm.world.factions[key] == nil and faction_keys[key] then
        rpfm.faction { key = key };
    end;
    return rpfm.world.factions[key];
end;

local function region_by_key(key)
    if rpfm.world.regions[key] == nil and region_keys[key] then
        rpfm.region { key = key };
    end;
    return rpfm.world.regions[key];
end;

local region_manager = rpfm.object("REGION_MANAGER_SCRIPT_INTERFACE", {
    region_by_key = function(key) return region_by_key(key) or rpfm.null("REGION_SCRIPT_INTERFACE") end,
    region_list = function() return rpfm.list("REGION_LIST_SCRIPT_INTERFACE", ordered(rpfm.world.region_order, rpfm.world.regions)) end,
});

local world = rpfm.object("WORLD_SCRIPT_INTERFACE", {
    faction_by_key = function(key) return faction_by_key(key) or rpfm.null("FACTION_SCRIPT_INTERFACE") end,
    faction_exists = function(key) return faction_by_key(key) ~= nil end,
    faction_list = function() return rpfm.list("FACTION_LIST_SCRIPT_INTERFACE", ordered(rpfm.world.faction_order, rpfm.world.factions)) end,
    region_manager = region_manager,
});

local model = rpfm.object("MODEL_SCRIPT_INTERFACE", {
    world = world,
    is_ready_for_script_access = true,
    is_multiplayer = false,
    turn_number = function() return rpfm.world.turn end,
});

local game_interface = rpfm.object("cm", {
    model = model,
});

--------------------------------------------------------------------------------
-- Globals the game provides.
--------------------------------------------------------------------------------

system = rpfm.library("system", { ClearRequiredFiles = function() end });

common = rpfm.library("common", {
    tweaker_value = "0",
    filesystem_lookup = function(folder, filter) return list_scripts(normalize_path(folder), filter or "*.*") end,
});

ScriptedValueRegistry = { new = function() return rpfm.object("ScriptedValueRegistry") end };
effect = rpfm.library("effect");
UIComponent = function() return rpfm.object("uicomponent") end;
GAME = function() return game_interface end;

print = function(...)
    local parts = {};
    for i = 1, select("#", ...) do
        table.insert(parts, tostring((select(i, ...))));
    end;
    log(table.concat(parts, "\t"));
end;

-- Output channels. The game has one per tab of its script debug console.
out = {};
for _, channel in ipairs({ "out", "design", "invasions", "narrative", "ui", "chaos", "interventions", "savegame", "help_pages", "grudges", "scripted_tours", "traits" }) do
    local prefix = channel == "out" and "" or "[" .. channel .. "] ";
    out[channel] = function(text) log(prefix .. tostring(text)) end;
end;

--------------------------------------------------------------------------------
-- Test API.
--------------------------------------------------------------------------------

-- Registers a test.
function rpfm.test(name, test_function)
    table.insert(rpfm.tests, { name = tostring(name), test_function = test_function });
end;

-- Creates a faction and adds it to the world. `key` is its name; any other field is the value of the method with
-- the same name, like `is_human = true`. Lists, like `region_list`, can be plain arrays.
function rpfm.faction(fields)
    local methods = {};
    for field, value in pairs(fields or {}) do
        methods[field] = value;
    end;
    methods.name = methods.name or methods.key;
    methods.key = nil;

    local faction = rpfm.object("FACTION_SCRIPT_INTERFACE", methods);
    if type(methods.name) == "string" then
        if rpfm.world.factions[methods.name] == nil then
            table.insert(rpfm.world.faction_order, methods.name);
        end;
        rpfm.world.factions[methods.name] = faction;
    end;
    return faction;
end;

-- Creates a region and adds it to the world. Fields work like in `rpfm.faction`.
function rpfm.region(fields)
    local methods = {};
    for field, value in pairs(fields or {}) do
        methods[field] = value;
    end;
    methods.name = methods.name or methods.key;
    methods.key = nil;

    local region = rpfm.object("REGION_SCRIPT_INTERFACE", methods);
    if type(methods.name) == "string" then
        if rpfm.world.regions[methods.name] == nil then
            table.insert(rpfm.world.region_order, methods.name);
        end;
        rpfm.world.regions[methods.name] = region;
    end;
    return region;
end;

-- Creates a character. Fields are the values of the methods with the same name, like `faction = my_faction`.
function rpfm.character(fields)
    return rpfm.object("CHARACTER_SCRIPT_INTERFACE", fields);
end;

-- Triggers an event, like the game does. Accessors are the values returned by the methods of the context, like
-- `faction = my_faction`. `string` is a field instead, like on the contexts of UI events.
function rpfm.fire(event, accessors)
    local methods = {};
    local fields = { string = "" };
    for key, value in pairs(accessors or {}) do
        if key == "string" then
            fields[key] = value;
        else
            methods[key] = value;
        end;
    end;

    local context = rpfm.object(event .. " context", methods, fields, api_accessors[event] or {});
    local callbacks = events and events[event];
    if type(callbacks) ~= "table" then
        return;
    end;

    -- Copied, so callbacks registering or removing others don't change which ones run now.
    local current = {};
    for index, callback in ipairs(callbacks) do
        current[index] = callback;
    end;

    for _, callback in ipairs(current) do
        local ok, err = xpcall(function() callback(context) end, debug.traceback);
        if not ok then
            table.insert(rpfm.errors, "Error in a listener of " .. event .. ": " .. tostring(err));
        end;
    end;
end;

local function describe(value)
    if type(value) == "string" then
        return string.format("%q", value);
    end;
    return tostring(value);
end;

local function describe_args(args)
    local parts = {};
    for i = 1, args.n do
        table.insert(parts, describe(args[i]));
    end;
    return "(" .. table.concat(parts, ", ") .. ")";
end;

-- Returns the arguments of every call to a method on engine objects, optionally only on objects of a type.
function rpfm.calls_to(method, type_name)
    local found = {};
    for _, call in ipairs(rpfm.calls) do
        if call.method == method and (type_name == nil or call.type == type_name) then
            table.insert(found, call.args);
        end;
    end;
    return found;
end;

-- Fails unless a method was called with arguments starting with the given ones.
function rpfm.assert_called(method, ...)
    local expected = pack(...);
    local calls = rpfm.calls_to(method);
    for _, args in ipairs(calls) do
        local matches = true;
        for i = 1, expected.n do
            if args[i] ~= expected[i] then
                matches = false;
                break;
            end;
        end;
        if matches then
            return;
        end;
    end;

    local actual = {};
    for _, args in ipairs(calls) do
        table.insert(actual, method .. describe_args(args));
    end;
    error("expected a call to " .. method .. describe_args(expected) .. ", got " .. (#actual > 0 and table.concat(actual, ", ") or "no calls to it"), 2);
end;

-- Fails if a method was called.
function rpfm.assert_not_called(method)
    local calls = rpfm.calls_to(method);
    if #calls > 0 then
        error("expected no calls to " .. method .. ", got " .. #calls .. ", the first one being " .. method .. describe_args(calls[1]), 2);
    end;
end;

-- Fails unless both values are equal.
function rpfm.assert_equal(actual, expected, message)
    if actual ~= expected then
        error((message and message .. ": " or "") .. "expected " .. describe(expected) .. ", got " .. describe(actual), 2);
    end;
end;

--------------------------------------------------------------------------------
-- Entry points used by RPFM.
--------------------------------------------------------------------------------

-- Loads the script libraries, the vanilla scripts of a campaign if any, and the mods, then gets to the first tick.
function rpfm.__boot(campaign)
    CampaignName = campaign or "main_warhammer";

    local ok, err = xpcall(function()
        dofile("script/campaign_scripted.lua");
        if campaign then
            package.path = package.path .. ";script/campaign/" .. campaign .. "/?.lua;script/campaign/?.lua";
            dofile("script/campaign/" .. campaign .. "/scripting.lua");
        else
            load_script_libraries();

            -- Created by the shared campaign scripts, which are only loaded with the vanilla scripts of a campaign.
            uim = cm:get_campaign_ui_manager();
        end;
    end, debug.traceback);

    if not ok then
        table.insert(rpfm.errors, "Error loading the script libraries: " .. tostring(err));
        return;
    end;

    for _, event in ipairs({ "NewSession", "WorldCreated", "UICreated", "FirstTickAfterWorldCreated" }) do
        rpfm.fire(event);
    end;
end;

-- Runs a test, after resetting what the boot left behind.
function rpfm.__run_test(index)
    local test = rpfm.tests[index];
    rpfm.calls = {};
    rpfm.errors = {};
    rpfm.unmocked = {};
    return xpcall(test.test_function, debug.traceback);
end;
