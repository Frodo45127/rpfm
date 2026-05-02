# Translator

The Translator is RPFM's dedicated tool for translating mods. It's a structured editor on top of a Pack's loc data, plus an integration with the [Total War Translation Hub](https://github.com/Frodo45127/total_war_translation_hub) so the resulting translations can be shared and applied automatically by Runcher.

Open from **Tools → Translator** with a Pack open.

<!-- IMAGE: Translator window showing the loc keys list on the left, original (vanilla / source) text in the middle, and the translation field on the right. Status icons indicate translated / outdated / new. -->

## What it does

The Translator presents every translatable string in the Pack as a structured row, with:

- **Key** — the loc key.
- **Source text** — the original (typically English) text.
- **Translation** — your translated text.
- **Needs retranslation** — boolean. Set when the source text changed since the translation was last saved (the "outdated" state).
- **Removed** — boolean. Set when the key was once translated but the mod no longer contains it (the "unused" state). The translation is kept in the JSON so it can come back if the key reappears.

## The workflow

1. **Auto-translate from vanilla.** For keys that exist in vanilla loc data and are unchanged, the Translator will auto-translate them using the vanilla translations, leaving the modded or altered lines to be translated.
2. **Translate row by row**, or using one of the translation integrations.
3. **Generate the translated loc.** When you save, the Translator writes a translated `.loc` file into the Pack at the right path: `text/!!!!!!translated_locs.loc` for Warhammer 1 and newer (except Thrones of Britannia), or `text/localisation.loc` for Thrones of Britannia and older games. The translation works in-game immediately.
4. **Persist the translation as JSON.** The translation is also persisted to `<config>/translations_local/<game>/<pack>/<LANGUAGE>.json`. This is the file you contribute to the [Translation Hub](https://github.com/Frodo45127/total_war_translation_hub).

## Sharing through the Translation Hub

Once you've finished a translation:

1. Find the JSON in `<config>/translations_local/<game>/<pack>/`.
2. Submit it to the Translation Hub as an issue or PR.
3. Once accepted, any Runcher user with **Enable Translations** turned on will get the translation automatically applied at launch — without altering the mod, and with `outdated` lines silently ignored. No more "I installed a translation pack but the mod was updated and now half the lines are wrong."

For step-by-step instructions including screenshots, see the [Translating a mod tutorial](../tutorials/translating-a-mod.md).
