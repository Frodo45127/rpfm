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

This tests a mod script like this one, in `script/campaign/mod/kislev_gold.lua`:

```lua
core:add_listener(
    "kislev_gold",
    "FactionTurnStart",
    function(context) return context:faction():name() == "wh3_main_ksl_kislev" and context:faction():is_human() end,
    function(context) cm:treasury_mod(context:faction():name(), 500) end,
    true
)
```

### How a test runs

Every test gets its own, fresh game. For each one, RPFM:

1. Runs the top-level code of the test file, creating the factions, regions and characters it sets up.
2. Loads the game's script libraries, then the mods of your open Packs (`script/campaign/mod/`), like the game does.
3. Triggers `NewSession`, `WorldCreated`, `UICreated` and `FirstTickAfterWorldCreated`, so your mods' listeners are registered and their first-tick code has run.
4. Runs the test's function.

Nothing done by a test is left behind for the next one: globals, listeners, timers and recorded calls all start from scratch.

## The test API

### Setting up the world

| Function | What it does |
|----------|--------------|
| `rpfm.test(name, function)` | Registers a test. |
| `rpfm.faction { key = ..., ... }` | Creates a faction. Any other field is what the method with the same name returns, like `is_human = true` or `region_list = { my_region }`. |
| `rpfm.region { key = ..., ... }` | Creates a region, with fields like `rpfm.faction`. |
| `rpfm.character { ... }` | Creates a character, like `rpfm.character { faction = kislev }`. |
| `rpfm.mock(object, method, value)` | Changes what a method of a faction, region or other game object returns, like `rpfm.mock(my_region, "owning_faction", kislev)`. `value` can also be a function, called with the arguments of each call. |

Fields can be plain values, other objects, or functions called with the arguments of each call. Methods returning lists, like `region_list` or `character_list`, can be given plain arrays: they're turned into the list the game would return, with `num_items`, `is_empty` and `item_at`.

```lua
local kiev = rpfm.region { key = "wh3_main_combi_region_kislev", is_province_capital = true }
local katarin = rpfm.character { character_subtype_key = "wh3_main_ksl_katarin", rank = 10 }

local kislev = rpfm.faction {
    key = "wh3_main_ksl_kislev",
    is_human = true,
    treasury = 2000,
    region_list = { kiev },
    character_list = { katarin },

    -- Called with the arguments the script passed.
    has_effect_bundle = function(bundle) return bundle == "my_bundle" end,
}

rpfm.mock(kiev, "owning_faction", kislev)
rpfm.mock(katarin, "faction", kislev)
```

Factions and regions from the game's database exist even if your test doesn't create them, so scripts looking them up by key, like with `cm:get_faction`, find them. Anything a test doesn't set up returns a neutral value of the type the scripting docs say, like `false`, `0`, an empty string, or a *null* object.

### Making things happen

| Function | What it does |
|----------|--------------|
| `rpfm.fire(event, { ... })` | Triggers an event, like the game does. The table has what the event's context returns, like `{ faction = kislev }`. |
| `rpfm.advance_time(seconds)` | Moves game time forward, running the timers that finish in that time, like the ones from `cm:callback`. |
| `rpfm.end_turn()` | Plays a full round: every faction, in the order they were created, starts and ends its turn, with the turn events of its regions and characters. |

`rpfm.end_turn()` also moves the turn number forward, and triggers these events, in this order:

1. `WorldStartRound`.
2. `FactionRoundStart`, for every faction.
3. For each faction: `FactionTurnStart`, `RegionTurnStart` for each of its regions, `CharacterTurnStart` for each of its characters, `FactionBeginTurnPhaseNormal`, `FactionAboutToEndTurn`, `CharacterTurnEnd` for each character, `RegionTurnEnd` for each region, and `FactionTurnEnd`.

Only the regions and characters given to the faction's `region_list` and `character_list` get their events.

### Checking the results

| Function | What it does |
|----------|--------------|
| `rpfm.assert_called(method, ...)` | Fails unless your scripts called the game's `method` with those arguments, like `rpfm.assert_called("treasury_mod", "wh3_main_ksl_kislev", 500)`. |
| `rpfm.assert_not_called(method)` | Fails if your scripts called the game's `method`. |
| `rpfm.calls_to(method[, type])` | Returns the arguments of every call to the game's `method`, for your own checks. `type` limits it to calls on objects of that type, like `"FACTION_SCRIPT_INTERFACE"`. |
| `rpfm.assert_equal(actual, expected[, message])` | Fails unless both values are equal. `message` is shown in front of the error, to tell which check failed. |

`rpfm.assert_called` only checks the arguments you give it, so `rpfm.assert_called("treasury_mod", "wh3_main_ksl_kislev")` passes whatever the amount was. When it fails, it lists the calls that were made instead:

```
expected a call to treasury_mod("wh3_main_ksl_kislev", 500), got treasury_mod("wh3_main_ksl_kislev", 250)
```

Your scripts' own globals are there too, so tests can inspect and change them directly, like enabling a single feature of your mod before checking it. Any other Lua error in a test, like one from `error()` or `assert()`, fails it too.

## Examples

### Timers

Scripts using `cm:callback` or `cm:repeat_callback` only run their callbacks when game time moves forward:

```lua
-- The mod: warns the player 10 seconds after the campaign starts, and every 30 seconds after that.
my_mod_warnings = 0

cm:add_first_tick_callback(function()
    cm:callback(function() my_mod_warnings = my_mod_warnings + 1 end, 10)
    cm:repeat_callback(function() my_mod_warnings = my_mod_warnings + 1 end, 30, "my_mod_reminder")
end)
```

```lua
-- The test.
rpfm.test("warns after 10 seconds, then every 30", function()
    rpfm.advance_time(5)
    rpfm.assert_equal(my_mod_warnings, 0, "after 5 seconds")

    rpfm.advance_time(5)
    rpfm.assert_equal(my_mod_warnings, 1, "after 10 seconds")

    rpfm.advance_time(80)
    rpfm.assert_equal(my_mod_warnings, 4, "after 90 seconds")
end)
```

For timers reported through events, `rpfm.advance_time` triggers `TimeTrigger` with the timer's id in `context.string`, like the game does.

### Several turns

```lua
local kislev = rpfm.faction { key = "wh3_main_ksl_kislev", is_human = true }

rpfm.test("pays every turn, for three turns", function()
    rpfm.end_turn()
    rpfm.end_turn()
    rpfm.end_turn()

    rpfm.assert_equal(#rpfm.calls_to("treasury_mod"), 3)
    rpfm.assert_equal(cm:model():turn_number(), 4)
end)
```

### Changing the world during a test

`rpfm.mock` can be used inside a test, to change the world between events. Here, a mod that should only reward a faction once it owns a region:

```lua
local kislev = rpfm.faction { key = "wh3_main_ksl_kislev", is_human = true }
local kiev = rpfm.region { key = "wh3_main_combi_region_kislev" }

rpfm.test("rewards Kislev once it owns Kislev", function()
    rpfm.fire("FactionTurnStart", { faction = kislev })
    rpfm.assert_not_called("apply_effect_bundle")

    rpfm.mock(kiev, "owning_faction", kislev)
    rpfm.mock(kislev, "region_list", { kiev })

    rpfm.fire("FactionTurnStart", { faction = kislev })
    rpfm.assert_called("apply_effect_bundle", "my_reward_bundle", "wh3_main_ksl_kislev")
end)
```

### Checking the calls yourself

`rpfm.calls_to` gives you every call, for checks the assertions can't do, like the order or the number of calls:

```lua
local kislev = rpfm.faction { key = "wh3_main_ksl_kislev", is_human = true }
local ungol = rpfm.faction { key = "wh3_main_ksl_ungol", is_human = true }

rpfm.test("pays every human faction once per round, in turn order", function()
    rpfm.end_turn()

    local payments = rpfm.calls_to("treasury_mod")
    rpfm.assert_equal(#payments, 2, "number of payments")

    -- Each entry has the arguments of a call, in order.
    rpfm.assert_equal(payments[1][1], "wh3_main_ksl_kislev")
    rpfm.assert_equal(payments[2][1], "wh3_main_ksl_ungol")
end)
```

Calls are recorded when they reach the game, so library functions like `cm:treasury_mod` show up under the name of the game function they end up calling, which is usually the same.

### UI events

UI events, like clicks, carry the name of the component in `context.string`, which can be set like any other accessor:

```lua
rpfm.test("clicking my button opens my panel", function()
    rpfm.fire("ComponentLClickUp", { string = "my_mod_button" })
    rpfm.assert_equal(my_mod.panel_open, true)
end)
```

### Testing a single feature

As tests see your scripts' globals, they can reach inside your mod, like to change its settings or call its functions directly:

```lua
rpfm.test("the difficulty setting doubles the gold", function()
    my_mod.settings.difficulty = "hard"
    my_mod.pay_faction(kislev)
    rpfm.assert_called("treasury_mod", "wh3_main_ksl_kislev", 1000)
end)
```

## Running tests

Select one or more test files in the [Pack tree](./pack-tree.md), right-click them and choose **Run Lua Tests**. The results window lists every test, with:

- **Errors**: why it failed. Tests also fail when one of your scripts errors while the test runs, or while the game starts.
- **Undocumented calls**: game functions your scripts called that the scripting docs don't describe. They return placeholder values, so a test depending on them may not match the game.
- **Output**: everything the scripts printed, with `print` or `out`. For long runs, only the latest lines are kept.

Errors of the game's own scripts while starting are listed separately, and never fail your tests: they usually mean the fake game lacks something those scripts need.

Tests can also be run from tools and AI agents connected to [the server](../server/overview.md), through the [`LuaRunTests`](../server/ws-commands.md#luaruntests) command or the `lua_run_tests` [MCP tool](../server/mcp.md). These can also load the vanilla scripts of a campaign, like `main_warhammer`, by passing its name. That's closer to the game, but slower, and more likely to hit parts of the game the harness doesn't fake. Running tests from the Pack tree only loads the script libraries and your mods.

## Limitations

The game is faked, not run, so keep in mind that:

- Only what your tests set up exists, besides the factions and regions of the game's database. Armies don't move, battles don't happen, and methods like `treasury_mod` are recorded but don't change what `treasury` returns, unless your test mocks it.
- Methods the scripting docs don't have return placeholder values, and are listed under **Undocumented calls**. Asking a documented object for a method it doesn't have returns `nil`, like in the game.
- Scripts can't run programs, load DLLs, or touch your files. Files they write are kept in memory, and can be read back during the same test.
- `string.len`, `string.sub` and `string.find` count characters instead of bytes, and `string.find` doesn't use patterns, like in the game. The original versions are available as `string.len_lua`, `string.sub_lua` and `string.find_lua`.
- Scripts running for too long, like when stuck in a loop, are stopped, failing the test.
