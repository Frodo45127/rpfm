# Translating a mod

There are three approaches to translating a Total War mod. RPFM supports all of them, but the third — using the integrated Translator — is the recommended one.

## The simple way

The one new modders tend to do, because it's the easiest:

1. Open the Pack.
2. Open the Loc files.
3. Manually translate the loc values.
4. Save the Pack.

It's not recommended. When the mod gets an update, you have to retranslate everything, or hunt down the new and changed lines and translate just those. Both flows are painful and error-prone, and you usually end up with a translation that "works but is partially outdated".

## The TSV way

The one people that have been hit by the simple-way problems tend to do:

1. Open the Pack.
2. Export every Loc to TSV.
3. Use a translation tool to work on the TSV.
4. Import the locs back.
5. Save the Pack.

There are many variants of this — from "autotranslate everything with Google" (please proofread) to advanced workflows that isolate new / changed lines and only update those.

Translations are easier to update and easier to keep tidy in version control, but the workflow takes more setup time.

## The new way: the Translator

RPFM ships with an integrated Translator that handles every painful part of the other two:

1. Open the Pack you want to translate.
2. **Tools → Translator**.
3. Translate the lines marked **Needs Retranslation**.
4. Hit **Accept**.
5. Save the Pack.

The Translator does the heavy lifting for you and fixes problems that have historically plagued translations:

- Detects lines unchanged from the vanilla English text and translates them automatically using the vanilla translation in the target language.
- When translating a submod, reuses translations from parent mods if they exist.
- When updating a translation, marks lines that have been removed, lines that have been altered, and lines that are new.
- Saves the translation in a structured form that's easy to share and collaborate on.

This means: no time wasted updating a translation (open the updated mod in the Translator and accept), no lines stuck on old translations after an update, no terminology drift across files.

## How to use the Translator

<!-- IMAGE: Translator window with the loc list on the left, the original / translated panes in the middle/right, and the auto-translation behaviour selectors visible. -->

The translator window has two halves.

### Left side: the loc list

| Column                | Meaning |
|-----------------------|---------|
| **Key**               | Loc key |
| **Needs Retranslation?** | Checked when the line needs work — either it's new, or the source text has changed since the previous translation. |
| **Removed**           | The line is no longer in the mod. The translator hides these by default. |
| **Original Value**    | The source text (usually English). |
| **Translated Value**  | Your translation. |

### Right side: the editor

<!-- IMAGE: The right side of the translator: language indicator at the top, auto-translation behaviour pickers, original-value pair (raw + formatted), translated-value pair (raw + formatted) with the navigation buttons. -->

At the top: a short instruction line and the target language (picked automatically based on the language you have the game on).

**Auto-translation behaviour** — what should happen when you select a new line:

- **Auto-translate with DeepL** — uses [DeepL](https://www.deepl.com/) for high-quality machine translation. Requires a DeepL API key set in **PackFile → Settings → AI**.
- **Auto-translate with ChatGPT** — uses ChatGPT, with optional context. Quality varies. Requires an OpenAI API key in the same AI pane.
- **Auto-translate with Google Translate** — mediocre but free.
- **Copy Source Value** — leaves the source text in place; useful for terms that don't translate.
- **Empty Translated Value** — does nothing on selection; you write the translation from scratch.

When you save the current line:

- **Replicate Edit in Pre-Translated Lines** — also overwrite previously-translated lines that share the same source text.
- **Only Edit the Current Line** — disable cross-line replication entirely.

Below the behaviour selectors: the **Original Value** (raw + formatted preview) and the **Translated Value** (raw + formatted preview, with prev/next buttons).

The pattern is: select a line, the right side loads it, you edit, you select the next line, repeat. When you're done, **Accept** writes the translation to two places:

- **Inside the open Pack**, as `text/!!!!!!translated_locs.loc` (or `text/localisation.loc` for older games), so the translation works in-game immediately.
- **On disk**, as `<RPFM config>/translations_local/<game>/<pack>/<LANGUAGE>.json`. This is the file you share.

## Sharing a translation

Submit the JSON to the [Total War Translation Hub](https://github.com/Frodo45127/total_war_translation_hub) as an issue or a PR.

Why share?

- The Translator automatically searches the Hub for existing translations of mods you open. If someone translated this mod a year ago, you don't start from scratch — you pick up where they left off.
- Runcher (and any other launcher using TWPatcher) automatically pulls the Hub at launch and applies translations transparently — fixing the dreaded "no text when not English" bug along the way.

So sharing makes the translation future-proof and lets others use it without juggling extra packs or worrying about parent-mod compatibility.
