# Lua tests

> **Experimental.** Lua tests are new, and the way they emulate the game may still change in future versions.

Lua tests let you check what your mod's scripts do without starting the game. RPFM runs your scripts together with the game's own script libraries, fakes the game around them, and tells you whether they did what you expected.

Lua tests need the game's Assembly Kit, as RPFM uses its scripting docs to know what the game's functions return. Only campaign scripts are supported for now.

## Writing a test file

A test file is a `.lua` file that describes the situation to test, and what your scripts should do in it. Keep test files outside `script/`, for example in a `tests/` folder of your Pack, so the game never loads them:

```lua
-- Code outside tests runs before the game starts: set up the factions and regions your scripts need.
local kislev = rpfm.faction { key = "wh3_main_ksl_kislev", is_human = true }
local empire = rpfm.faction { key = "wh_main_emp_empire", is_human = false }

rpfm.test("gives gold to Kislev at the start of its turn", function()
    rpfm.fire("FactionTurnStart", { faction = kislev })
    rpfm.assert_called("treasury_mod", "wh3_main_ksl_kislev", 500)
end)

rpfm.test("doesn't give gold to AI factions", function()
    rpfm.fire("FactionTurnStart", { faction = empire })
    rpfm.assert_not_called("treasury_mod")
end)
```

Each test starts from a fresh game: the script libraries, the mods of your open Packs (`script/campaign/mod/`) and the first tick have already run, and nothing done by other tests is left behind.

## The test API

| Function | What it does |
|----------|--------------|
| `rpfm.test(name, function)` | Registers a test. |
| `rpfm.faction { key = ..., ... }` | Creates a faction. Any other field is what the method with the same name returns, like `is_human = true` or `region_list = { my_region }`. |
| `rpfm.region { key = ..., ... }` | Creates a region, with fields like `rpfm.faction`. |
| `rpfm.character { ... }` | Creates a character, like `rpfm.character { faction = kislev }`. |
| `rpfm.mock(object, method, value)` | Changes what a method of a faction, region or other game object returns, like `rpfm.mock(my_region, "owning_faction", kislev)`. `value` can also be a function, called with the arguments of each call. |
| `rpfm.fire(event, { ... })` | Triggers an event, like the game does. The table has what the event's context returns, like `{ faction = kislev }`. |
| `rpfm.advance_time(seconds)` | Moves game time forward, running the timers that finish in that time, like the ones from `cm:callback`. |
| `rpfm.end_turn()` | Plays a full round: every faction, in the order they were created, starts and ends its turn, with the turn events of its regions and characters. |
| `rpfm.assert_called(method, ...)` | Fails unless your scripts called the game's `method` with those arguments, like `rpfm.assert_called("treasury_mod", "wh3_main_ksl_kislev", 500)`. |
| `rpfm.assert_not_called(method)` | Fails if your scripts called the game's `method`. |
| `rpfm.calls_to(method)` | Returns the arguments of every call to the game's `method`, for your own checks. |
| `rpfm.assert_equal(actual, expected)` | Fails unless both values are equal. |

Your scripts' own globals are there too, so tests can inspect and change them directly, like enabling a single feature of your mod before checking it.

Factions and regions from the game's database exist even if your test doesn't create them, so scripts looking them up by key find them. Anything a test doesn't set up returns a neutral value of the type the scripting docs say, like `false`, `0`, an empty string, or a *null* object.

## Running tests

Select one or more test files in the [Pack tree](./pack-tree.md), right-click them and choose **Run Lua Tests**. The results window lists every test, with:

- **Errors**: why it failed. Tests also fail when one of your scripts errors while the test runs, or while the game starts.
- **Undocumented calls**: game functions your scripts called that the scripting docs don't describe. They return placeholder values, so a test depending on them may not match the game.
- **Output**: everything the scripts printed. For long runs, only the latest lines are kept.

Errors of the game's own scripts while starting are listed separately, and never fail your tests: they usually mean the fake game lacks something those scripts need.
