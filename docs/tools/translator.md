# Translator

The Translator is RPFM's dedicated tool for translating mods. It's a structured editor on top of a Pack's loc data, plus an integration with the [Total War Translation Hub](https://github.com/Frodo45127/total_war_translation_hub) so the resulting translations can be shared and applied automatically by Runcher.

Open from **Tools → Translator** with a Pack open.

<!-- IMAGE: Translator window showing the loc keys list on the left, original (vanilla / source) text in the middle, and the translation field on the right. Status icons indicate translated / outdated / new. -->

## What it does

The Translator presents every translatable string in the Pack as a structured row, with:

- **Key** — the loc key.
- **Source text** (`src`) — the original (typically English) text.
- **Translation** (`dst`) — your translated text.
- **Needs retranslation** (`retr`) — boolean. Set when the source text changed since the translation was last saved (the "outdated" state).
- **Removed** (`rem`) — boolean. Set when the key was once translated but the mod no longer contains it (the "unused" state). The translation is kept in the JSON so it can come back if the key reappears.
- **Auto-translated** (`aut`) — boolean. Set when the Translator filled the row automatically (from vanilla loc data or a translation service) and the user hasn't reviewed it yet. Editing the translation clears the flag.

The JSON file also records its format `version`. `0` is the legacy format (compatible with older Translation Hub tooling); `1` is the current format, which adds the `aut` flag and the source language (`src_lang`). It's selectable from the translator UI for English sources, and new translations default to `1`. Switching it rewrites the file in the chosen format on the next save.

## The workflow

1. **Auto-translate from vanilla.** For keys that exist in vanilla loc data and are unchanged, the Translator will auto-translate them using the vanilla translations, leaving the modded or altered lines to be translated.
2. **Translate row by row**, or using one of the translation integrations. The batch button in the toolbar auto-translates every outdated line at once with DeepL, AI or Google Translate. By default it only fills lines with no translation; lines with an outdated translation are left for you to fix, unless **Overwrite existing outdated translations** is checked in its menu. Batch results are marked as auto-translated so you can review them. A progress dialog shows how far along it is and lets you cancel it. Requests are spaced out, and when a service rate-limits them RPFM slows down and retries. At the end, a results dialog shows how many lines were translated, failed, or were left unprocessed, with the failed lines and their errors in its details.
3. **Generate the translated loc.** When you save, the Translator writes a translated `.loc` file into the Pack at the right path: `text/!!!!!!translated_locs.loc` for Warhammer 1 and newer (except Thrones of Britannia), or `text/localisation.loc` for Thrones of Britannia and older games. The translation works in-game immediately.
4. **Persist the translation as JSON.** The translation is also persisted to `<config>/translations_local/<game>/<pack>/<SOURCE>-<LANGUAGE>.json` (or `<LANGUAGE>.json` for format `0`). This is the file you contribute to the [Translation Hub](https://github.com/Frodo45127/total_war_translation_hub).

## Translating from a language other than English

When opening the Translator, you pick the language the mod is written in. The last one you picked is preselected.

The Translation Hub only ships vanilla texts in English. For any other source language, RPFM extracts them from the game's own `local_<language>` packs the first time you pick that language. It saves them to `<config>/translations_local/<game>/vanilla_<language>.tsv`, so they stay available after the game no longer has that language installed.

Many games only install the language they're set to. If the source language isn't installed, the Translator warns you that vanilla lines won't be auto-translated. To get those texts:

1. Change the game's language to the source language in Steam (**Properties → General → Language**) and let Steam download it.
2. Open the Translator with that language as source. RPFM saves its vanilla texts and tells you when it's done.
3. Switch the game back to your language, and open the Translator again.

## Sharing through the Translation Hub

Once you've finished a translation:

1. Find the JSON in `<config>/translations_local/<game>/<pack>/`.
2. Submit it to the Translation Hub as an issue or PR.
3. Once accepted, any Runcher user with **Enable Translations** turned on will get the translation automatically applied at launch — without altering the mod, and with `outdated` lines silently ignored. No more "I installed a translation pack but the mod was updated and now half the lines are wrong."

For step-by-step instructions including screenshots, see the [Translating a mod tutorial](../tutorials/translating-a-mod.md).
