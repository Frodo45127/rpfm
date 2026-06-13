# Plugin scripts

> **Experimental.** Plugin scripts are a new, experimental feature. The way scripts are invoked and how data is handed to them may still change in future versions.

Plugin scripts let you run your own **Python** or **Lua** scripts against the files you have selected in the [Pack tree](./pack-tree.md). They're meant for bulk, repetitive edits that would be tedious to do by hand — prefixing every Loc entry, rewriting a column across many tables, batch-renaming references, and so on.

## Installing a script

Drop a `.py` (Python) or `.lua` (Lua) file into the `scripts` folder inside RPFM's [config folder](../reference/settings.md#where-settings-live-on-disk). RPFM creates the folder for you the first time it's needed; you can also reach the config folder quickly via **Game Selected → Open RPFM's Config Folder**.

The interpreter is chosen from the file extension, so `python` (for `.py`) or `lua` (for `.lua`) must be available on your system `PATH`. The bundled Lua examples run on Lua 5.1 through 5.4.

You'll find ready-to-use examples in the `examples/plugin_scripts/` folder of the RPFM repository.

## Running a script

1. Select one or more files/folders in the Pack tree.
2. Right-click and open the **Run Script** submenu. It lists every script currently in your `scripts` folder (or **No scripts found** if the folder is empty).
3. Pick a script. RPFM runs it and reads the results back into the Pack.

## How a script is run

1. The files you selected are **extracted to a temporary folder** that mirrors their in-Pack layout:
   - **DB and Loc tables** are written as **TSV** (tab-separated, quoting disabled), exactly like the normal *Extract* action.
   - **Everything else** is written as **raw binary**.
2. Your script is launched with the extracted file paths as arguments (`python <script> <paths...>` or `lua <script> <paths...>`).
3. When the script finishes, RPFM reads the (possibly modified) files back into the Pack. Open views of changed files are reloaded and the Pack is marked as modified.

Files the script **deletes** are left untouched in the Pack, and files it **creates anew** are ignored — only the originally-selected files are read back.

## The TSV format

When a script receives a DB table or Loc file, it gets a TSV with **two header rows** before the data:

```
key<TAB>text<TAB>tooltip          <- column names
#Loc;1;text/db/mymod.loc<TAB><TAB> <- metadata: #table_type;version;path
mymod_key1<TAB>Hello world<TAB>false
```

Rows are `\r\n`-terminated and values are never quoted (so they can't contain tabs or newlines). **Keep the two header rows intact** when editing, or RPFM won't be able to read the table back.

## Debugging

Every run writes the script's output to `last_run.log` inside the `scripts` folder (overwritten each run). It contains the script path, exit status, and the full stdout/stderr. If a script exits with a non-zero status, RPFM also shows its stderr in a dialog. The same output is logged to the RPFM Server terminal.
