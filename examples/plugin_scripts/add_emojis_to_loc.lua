-- RPFM plugin example: wrap every Loc entry's text with an emoji.
--
-- RPFM extracts the files you selected to a temp folder and passes their paths as arguments.
-- DB and Loc tables arrive as TSV (tab-separated, quoting disabled), everything else as raw
-- binary. This script targets Loc TSVs and adds an emoji to the start and end of every `text`
-- value; RPFM then reads the result back into the Pack.
--
-- Drop this file in RPFM's config `scripts` folder, then right-click one or more loc files in
-- the contents tree and pick "Run Script". RPFM selects the interpreter from the `.lua`
-- extension, so `lua` must be on PATH. Works on Lua 5.1 through 5.4 (the TSV is UTF-8 and we
-- only splice bytes, so no unicode library is needed).

-- The emoji as its raw UTF-8 bytes (U+1F525 "fire" = F0 9F 94 A5), so this file carries no
-- literal emoji glyph and needs no utf8 library.
local EMOJI = "\240\159\148\165"
local PREFIX = EMOJI .. " "
local SUFFIX = " " .. EMOJI

-- Split a string on a single-character separator, keeping empty fields.
local function split(text, separator)
    local fields = {}
    local start = 1
    while true do
        local pos = string.find(text, separator, start, true)
        if not pos then
            fields[#fields + 1] = string.sub(text, start)
            break
        end
        fields[#fields + 1] = string.sub(text, start, pos - 1)
        start = pos + 1
    end
    return fields
end

-- The metadata row looks like "#Loc;1;path" (or "#Loc PackedFile;..." for older exports).
local function is_loc_metadata(line)
    local type_name = line:gsub("^#", ""):match("^(.-);")
    return type_name == "Loc" or type_name == "Loc PackedFile"
end

local function process_tsv(path)
    local handle = assert(io.open(path, "rb"))
    local content = handle:read("*a")
    handle:close()

    -- Split into lines, tolerating both '\n' and '\r\n'.
    local lines = {}
    for line in (content .. "\n"):gmatch("(.-)\n") do
        lines[#lines + 1] = (line:gsub("\r$", ""))
    end
    while #lines > 0 and lines[#lines] == "" do
        lines[#lines] = nil
    end

    -- A valid table TSV always has a column-names row and a metadata row.
    if #lines < 2 then
        return
    end

    if not is_loc_metadata(lines[2]) then
        print("Skipping non-Loc TSV: " .. path)
        return
    end

    local text_index = nil
    for index, name in ipairs(split(lines[1], "\t")) do
        if name == "text" then
            text_index = index
            break
        end
    end
    if not text_index then
        return
    end

    -- Lines 1 (column names) and 2 (metadata) are kept verbatim; the rest are data rows.
    local updated = 0
    for i = 3, #lines do
        if lines[i] ~= "" then
            local cells = split(lines[i], "\t")
            if cells[text_index] then
                cells[text_index] = PREFIX .. cells[text_index] .. SUFFIX
                lines[i] = table.concat(cells, "\t")
                updated = updated + 1
            end
        end
    end

    -- RPFM writes '\r\n'-terminated rows (including the last one), so we match that.
    handle = assert(io.open(path, "wb"))
    handle:write(table.concat(lines, "\r\n") .. "\r\n")
    handle:close()

    print(string.format("Updated %d entries in %s", updated, path))
end

for i = 1, #arg do
    local path = arg[i]
    if path:lower():match("%.tsv$") then
        process_tsv(path)
    end
end
