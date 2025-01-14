# How To Translate A Mod

There are three ways to translate a mod: **the simple way, the advanced way, and the new way**.

### The Simple Way

It's the one new modders tend to do, because it's the easier one:

- Open the Pack.
- Open the Loc files.
- Manually translate the loc values.
- Save the Pack.

It's not recomended because when the mod gets an update, you have to retranslate everything, or figure out a way to find the new/changed lines, translate those, then update your translation mod. And this can be a pain, and depending on how you do it, you can miss line changes leaving your translation ***working but partially outdated***.

### The Advanced Way

It's the one people that have been hit by the problems of the simple way tend to do:

- Open the Pack.
- Extract all locs to TSV.
- Use some software to help you translate the files.
- Import the locs back.
- Save the Pack.

There are many variants of this, ranging from simple **autotranslate everything with google** (if you do this WITHOUT PROOFREADING AND FIXING THE TRANSLATION I wish you a really painful death), to advanced workflows to isolate new/changed lines and only update those.

The pros of this is **translations are more easily updated** and it's **easier to keep track of the changes**. The cons are... that **it's more time-consuming**.

### The New Way

The new way is using RPFM's integrated Translator. It's really simple:

- Open the Pack.
- Open the Translator (Tools/Translator).
- Translate the lines left untranslated.
- Hit Accept.
- Save the Pack.

Why is this approach the recommended one? Because **the translator does a lot of the heavy work** you usually need to do:

- It detects lines which are unchanged vs the vanilla english translation, and translates them automatically.
- If multiple lines share the same text, it detects them and applies the same translation to all of them.
- When translating, it provides you with the google translation to use it as a start, speeding up translations as you only need to proofread and fix it.
- If you're updating a translation:
    - It detects lines which have been removed from the mod and marks them so you don't need to translate them.
    - It detects lines which have been altered on the mod, so you can update their translation.
    - It detects lines which are new, and promps you to translate them.

Additionally, it allows you to **import translations made in any of the other two ways** into it. And the translator generates translation json files which can be used by launchers to **automatically apply translations on the fly** to mods on your load order, like Runcher does with its "Enable Translations" feature.

So basically, it's like doing it the Simple Way, but faster and with the benefits of the Advanced Way.
