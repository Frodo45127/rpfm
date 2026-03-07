# Diagnostics Panel

![Tell me, doctor, do I have the coronavirus?](./images/diagnostics.png)

`Diagnostics` panel allows you to quickly identify possible problems in your mod, so you can fix them before they're reflected in-game.

It's pretty simple. From left to right:
- `Check PackFile`: Performs a diagnostics check over the entire PackFile. If the relevant settings are enabled, this is done automatically on PackFile opening too.
- `Check Open PackedFile`: Performs a diagnostics check over the open PackedFiles, leaving the results of the other PackFiles as they are.
- `Error`: Enable showing error diagnostics.
- `Warning`: Enable showing warning diagnostics.
- `Info`: Enable showing info diagnostics.
- `Open PackedFiles Only`: Filter the diagnostics list to show only the diagnostics relevant to the open PackedFiles.
- `Show more filters`: Shows a toggleable list of per-diagnostic filter, for more granular filtering.

To know more about what each diagnostic means, hover the mouse over them and you'll get an explanation of what it means. Also, double-clicking them will led you to the relevant place where they are being detected.

Also, something to note: while most diagnostics are automatic (they work without any setup needed), lua diagnostics (diagnostics to check if db keys in script are correct) need some setup by the modder. Specifically, the modder needs to write the following comment in the line above the variable, replacing table_name and column_name with the name of the table and column where that key should be:
```lua
--@db table_name column_name [key=0,value=1]
```

Here are some examples with all the variable formats supported by these diagnostics:
```lua
--@db battles key
hb = "chs_wastes_mountain_a_01"

--@db battles key
hb = { "chs_wastes_mountain_a_01", "chs_wastes_mountain_a_01" }

--@db battles key 0,1 (this checks both, key and value)
hb = { "chs_wastes_mountain_a_01" = "chs_wastes_mountain_a_01", "chs_wastes_mountain_a_01" = "chs_wastes_mountain_a_01" }

--@db battles key 0 (this checks only the keys)
hb = { "chs_wastes_mountain_a_01" = "chs_wastes_mountain_a_01", "chs_wastes_mountain_a_01" = "chs_wastes_mountain_a_01" }

--@db battles key 1 (this checks only the values)
hb = { "chs_wastes_mountain_a_01" = "chs_wastes_mountain_a_01", "chs_wastes_mountain_a_01" = "chs_wastes_mountain_a_01" }

--@db battles key
hb = {
    "chs_wastes_mountain_a_01",
    "chs_wastes_mountain_a_01"
}

--@db battles key 0,1 (this checks both, key and value)
hb = {
    "chs_wastes_mountain_a_01" = "chs_wastes_mountain_a_01",
    "chs_wastes_mountain_a_01" = "chs_wastes_mountain_a_01"
}

--@db battles key 0 (this checks only the keys)
hb = {
    "chs_wastes_mountain_a_01" = "chs_wastes_mountain_a_01",
    "chs_wastes_mountain_a_01" = "chs_wastes_mountain_a_01"
}

--@db battles key 1 (this checks only the values)
hb = {
    "chs_wastes_mountain_a_01" = "chs_wastes_mountain_a_01",
    "chs_wastes_mountain_a_01" = "chs_wastes_mountain_a_01"
}

```
