# Road to 1.0:
### PackFile Management:
  - [x] Add File/Folder.
  - [x] Add File/Folder from PackFile.
  - [ ] Create new folder.
  - [ ] Create new table.
  - [ ] Create new Loc.
  - [x] Delete.
  - [x] Rename.
  - [x] Extract.
  - [ ] Open with external tool.
  - [ ] Copy file/folder.
  - [ ] Cut file/folder.
  - [ ] Paste file/folder.

### PackedFile Management:
  - [x] RigidModel:
    - [x] Rework current decoding process to be more reliable with multiple Lods.
  - [ ] Loc:
    - [x] Edit rows.
    - [x] Add rows.
    - [x] Delete rows.
    - [ ] Copy rows.
    - [ ] Cut rows.
    - [ ] Paste rows.
    - [x] Import from csv.
    - [x] Export to csv.
  - [ ] DB:
    - [ ] Table View:
      - [x] Edit rows.
      - [x] Add rows.
      - [x] Delete rows.
      - [x] Clone rows.
      - [ ] Copy rows.
      - [ ] Cut rows.
      - [ ] Paste rows.
      - [x] Import from csv.
      - [x] Export to csv.
    - [x] Decoder View:
      - [x] Fix the hex view.
      - [x] Decode any type of entry.
      - [x] Load old definitions of an undecoded table.
      - [x] Manipulate decoded columns.
      - [x] Update "First row decoded" field on column order change.
	- [ ] Others:
	  - [x] Decode simple text files (with syntax highlight if possible).
	  - [x] Show images in the program (png, jpeg, jpg, tga).
	  - [ ] Show dds files in the program.

### QoL Improvements:
  - [ ] All: Improve the general behavior of the program.
  - [ ] Pref: Block edition in Boot/Release/Patch packs.
  - [ ] Pref: Allow to disable Cross-Table dependencies to improve performance.
  - [ ] PackFile Management: Improve interaction in case of duplicate files while adding them to the PackFile.
  - [ ] PackFile Management: Open PackFile directly from data folder.
  - [ ] PackFile Management: Don't try to decode the files if the selection has been done with right-click (or a better alternative for this).
  - [ ] PackFile Management: Delete multiple selected things at the same time.
  - [ ] PackFile Management: Extract multiple selected things at the same time.
  - [ ] PackFile Management: Move the load of the DB schemas to background (so it doesn't lock the UI) and make it happen on opening the PackFile (so we load only the needed schema).
  - [ ] PackedFile Management: Make it so right-clicking a row doesn't unselect the rest.
  - [x] PackedFile Management: Make the "column name rewriting" on opening table, instead on creation (to work better with dependencies).
  - [x] PackedFile Management: Improve decoder hex-view... so it's not broken.

### General Improvements:
  - [x] Hotkeys.
  - [x] Cross-table dependencies for DB Tables.
  - [ ] Minimize DB files function (requires some... imaginative coding to not fill 4GB of ram on minimize).
  - [ ] Column filtering for DB tables (custom, not permanent).
  - [ ] Loc Entries edition integrated in DB Tables View.
  - [ ] First-start setup dialog.
  - [x] MyMod Feature.
  - [x] Update checker.
  - [ ] Auto-updater.
  - [ ] Full support (PackFiles, DB Tables and Loc files) for:
    - [x] Warhammer 2.
    - [x] Warhammer.
    - [ ] Attila.
    - [ ] Rome 2.
    - [ ] Thrones of Britannia (if 1.0 is not done when it gets released).

### Extra Improvements (Not needed for 1.0, but want to have them done at some point):
  - [ ] Patch Attila's cs2.parsed to Warhammer format (for custom buildings).
  - [ ] Automatic creation of prefabs from maps (to ease mapmaking).
  - [ ] Extra theming options.
  - [ ] LUA files autocompletion.
  - [x] Font size selection.
  - [x] Migrate filechooser dialogs to use native dialogs.
  - [ ] Pref: Remember custom column widths (not so sure if I'll manage to make this one... or if it'll be done, but to the list it goes).
  - [ ] Code Tests, to ensure I don't break something with a code change.... again.
  - [ ] Something to not load entire PackFiles to Ram (optional, as this causes problems seen in PFM),
  - [ ] Rework how the entire TreeView works (including better controls to fold/expand the folders).
  - [x] Allow to open a PackFile directly by associating it with RPFM.
