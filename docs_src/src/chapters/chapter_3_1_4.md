# `Special Stuff` Menu

![Because you are S.P.E.C.I.A.L!](./../images/image10.png)

This menu contains... special features implemented for specific games. Basically, any feature that **doesn't really fit in any other place** goes here. Here we have:
- `Patch SiegeAI`: used in Warhammer 1 & 2 for **creating siege maps that the AI can handle**. Basically, make your map with the stuff required for the AI to work, and then patch his PackFile with this.
- `Optimize PackFile`: reduces the size of your PackFile by *cleaning* your tables from data that's unchanged from the vanilla game. It also does the same for Loc PackedFiles, **if you have the game's language set to *English*** . For example, if you have a table where all rows but one are exactly the same as the ones in vanilla tables and another table that's a 1:1 copy of a vanilla table without changes, RPFM remove all the rows but the one you changed from the first table, and it'll remove the second table. This is meant to **improve compatibility with other mods** , and to reduce the size of the PackFile.
- `Generate PAK File`: generates a file from raw data from the Assembly Kit that allows RPFM to provide a ton of reference data from tables not in the game. Or easier to understand, if you use the dependency checker, you'll have **far fewer blue columns**. Doesn't work for Empire and Napoleon, yet.
