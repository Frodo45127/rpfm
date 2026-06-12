# RPFM plugin scripts

RPFM can run user scripts against files selected in the PackFile contents tree. Drop a `.py`
(Python) or `.lua` (Lua) script into the `scripts` folder inside RPFM's config folder, then
right-click one or more files/folders and pick **Run Script**.

## How a script is run

1. The files you selected are extracted to a temp folder that mirrors their in-pack layout.
   - **DB and Loc tables** are written as **TSV** (tab-separated, quoting disabled), exactly
     like the normal *Extract* action.
   - **Everything else** is written as **raw binary**.
2. Your script is launched with the extracted file paths as arguments (`python <script> <paths...>`
   or `lua <script> <paths...>`). The interpreter is chosen from the script extension, so
   `python` / `lua` must be on `PATH`.
3. When the script finishes, RPFM reads the (possibly modified) files back into the Pack. Open
   views of changed files are reloaded and the Pack is marked as modified.

Files the script deletes are left untouched in the Pack, and files it creates anew are ignored
(only the originally-selected files are read back).

## Debugging

Every run writes the script's output to `last_run.log` inside the `scripts` folder (overwritten
each run), containing the script path, exit status, and its full stdout/stderr. If a script
exits with a non-zero status, RPFM also shows its stderr in a dialog. The same output is logged
to the RPFM Server terminal (stderr at `warn` level).

## TSV format reminder

A table TSV has two header rows before the data:

```
key<TAB>text<TAB>tooltip          <- column names
#Loc;1;text/db/mymod.loc<TAB><TAB> <- metadata: #table_type;version;path
mymod_key1<TAB>Hello world<TAB>false
```

Rows are `\r\n`-terminated and values are never quoted (so they can't contain tabs/newlines).
Keep the two header rows intact when editing.

## Examples

- `add_emojis_to_loc.py` / `add_emojis_to_loc.lua` — wrap every Loc entry's `text` value with an
  emoji at the start and end. Both target Loc TSVs only and skip anything else. The Lua version
  runs on Lua 5.1 through 5.4.
