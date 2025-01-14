# How To Optimize Your Mod

First, why you should optimize your mod:

- **Size**: optimizing your mod removes non-needed data from it, resulting in **smaller Packs**.
- **Compatibility**: optimizing your mod removes duplicated db and loc rows, unchanged rows vs vanilla... These rows **tend to be left forgotten in mod packs** (even when the diagnostics will warn you about them!!!) **and end up causing compatibility issues** when another mod tries to modify them, and your mod accidentally overwrite them for no reason.
- **Multi-Language Support**: optimizing your mod removes unchanged loc entries. This is important because, depending on the game, these loc entries will overwrite the player's localisation files if said player doesn't have the game in english. This means **you're not changing these lines, but they'll turn to english for the players**. Which is bad.

Next, how to do it? You have two options:

- ***Special Stuff/Game/Optimize Pack***: optimizes the Pack but it doesn't save it, so you can review the optimized Pack.
- ***Pack/Save Pack for Release***: optimizes the Pack and saves it.

Last, when to do it? Before releasing the mod/update. And that's all.
