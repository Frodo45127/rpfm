# `MyMod` Menu

![MyMod.... more like OurMod....](./images/image7.png)

`MyMod` is a feature to help modders **keep their mod's data organized**. The system is almost a 1:1 clone of PFM's *MyMod* feature so it should be easy to use for veterans too.

For those new with the concept, remember that `MyMod` folder we set in the settings? When we create a `MyMod`, in that folder will be created a folder for the game the mod's for (if it didn't exist before), and inside that there will be a PackFile and a folder, both with the name of your mod. Each time you extract something from the PackFile, it'll be automatically extracted in his folder, **mirroring the structure it has in the PackFile**. For example, extracting a table will result in the table being extracted at `mymod_folder/db/table_name/file`._ Adding Files/Folders from the MyMod folder **will also add them mirroring the path they have**. For example, adding a file from `mymod_folder/db/table_name/file`_ will add the file in `PackFile/db/table_name/file`.

This makes easier to keep track of the mod files, and you can even **put that folder under .git**, or any other version control system, as you can have an unpacked mod that you can pack with a single click (well, a few clicks).

The `MyMod` Menu has the following buttons:
- `Open MyMod Folder`: Opens the `MyMod` folder in your default file explorer.
- `New MyMod`: It opens the `New MyMod` Dialog. It's explained under this list.
- `Delete Selected MyMod`: It deletes the currently selected `MyMod`. This cannot be undone, so you'll got a warning before doing it.
- `Install`: Copy the currently `MyMod` PackFile to the data folder of his game, so you can test your changes in an easy way. You can re-install the mod to test further changes.
- `Uninstall`: Removes the currently selected `MyMod` PackFile from the data folder of the game.
- `XXX/yourmod.pack`: Open your previously created `MyMod` to edit, delete, install,.... whatever you want, baby!

When we click on `New MyMod`, the following dialog will appear:

![I said OURMOD!!!!](./images/image8.png)

Here you can configure the name and game the mod is for. Once you're done, hit `Save` and your new `MyMod` will be created and opened.

And lastly, a couple of aclarations:
- To be able to use `Install/Uninstall` you need to have your `MyMod` open.
- Only `MyMod` PackFiles opened from `XXX/yourmod.pack` will enjoy the `MyMod` features, like keeping the paths when adding/extracting files. Manually opened `MyMod` PackFiles will be treated as regular PackFiles.
