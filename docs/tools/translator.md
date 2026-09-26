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

The JSON file also records its format `version`. `0` is the legacy format (compatible with older Translation Hub tooling); `1` is the current format, which adds the `aut` flag, the source language (`src_lang`), the list of `authors` (typed comma-separated in the **Authors** field) and the glossary. It's selectable from the translator UI for English sources, and new translations default to `1`. Switching it rewrites the file in the chosen format on the next save.

## The workflow

1. **Auto-translate from vanilla.** For keys that exist in vanilla loc data and are unchanged, the Translator will auto-translate them using the vanilla translations, leaving the modded or altered lines to be translated.
2. **Translate row by row**, or using one of the translation integrations. AI translations are told which game the mod is for, and to prefer the game's official translations for its terms and names. The batch button in the toolbar auto-translates every outdated line at once with DeepL, AI or Google Translate. By default it only fills lines with no translation; lines with an outdated translation are left for you to fix, unless **Overwrite existing outdated translations** is checked in its menu. Batch results are marked as auto-translated so you can review them. A progress dialog shows how far along it is and lets you cancel it. Requests are spaced out, and when a service rate-limits them RPFM slows down and retries. DeepL gets up to 50 lines per request, so it needs far fewer requests; if one of those requests fails, all its lines are reported as failed. At the end, a results dialog shows how many lines were translated, failed, or were left unprocessed, with the failed lines and their errors in its details.
3. **Generate the translated loc.** When you save, the Translator writes a translated `.loc` file into the Pack at the right path: `text/!!!!!!translated_locs.loc` for Warhammer 1 and newer (except Thrones of Britannia), or `text/localisation.loc` for Thrones of Britannia and older games. The translation works in-game immediately.
4. **Persist the translation as JSON.** The translation is also persisted to `<config>/translations_local/<game>/<pack>/<SOURCE>-<LANGUAGE>.json` (or `<LANGUAGE>.json` for format `0`). This is the file you contribute to the [Translation Hub](https://github.com/Frodo45127/total_war_translation_hub).

## Editing a line

**Clear translation** resets the selected line to untranslated. It has a configurable shortcut, unbound by default.

To keep the editor compact, the quick start instructions, the auto-translation settings (the ones applied when a line is selected) and the formatted in-game style previews of the source and translated text are hidden by default. Show them with the **Help**, **Auto-translate settings** and **Preview** toggles.

## Glossary

The glossary is a per-translation list of source terms and the translation you want used for each one, like faction or unit names. Open it with the **Glossary** button: it opens over the right side of the translator, so the lines stay visible for reference, and its close button hides it again. Edit it like any other table, with its toolbar, right-click menu or shortcuts. Entries without a source term are ignored.

When auto-translating, the glossary is used like this:

- **DeepL**: with the **Use a DeepL glossary** toolbar button enabled (the default), RPFM stores the glossary in your DeepL account and DeepL enforces its terms. There's one per Pack and language pair, named `RPFM <game>/<pack> <source>-<target> <fingerprint>`; it's reused while the glossary doesn't change, and replaced when it does. Entries without a translation are left out of it. If it can't be stored, or the button is disabled, the glossary is sent as a hint instead.
- **AI**: the glossary is included in the prompt, as a hint.
- **Google Translate**: doesn't support it.

Review glossary terms in hint-based results, as the service may not always follow them.

The glossary is saved with the translation, and only format `1` can store it. With format `0` the Glossary button is disabled.

## Translating from a language other than English

When opening the Translator, you pick the language the mod is written in. The last one you picked is preselected.

The Translation Hub only ships vanilla texts in English. For any other source language, RPFM extracts them from the game's own `local_<language>` packs the first time you pick that language. It saves them to `<config>/translations_local/<game>/vanilla_<language>.tsv`, so they stay available after the game no longer has that language installed.

Many games only install the language they're set to. If the source language isn't installed, the Translator warns you that vanilla lines won't be auto-translated. To get those texts:

1. Change the game's language to the source language in Steam (**Properties → General → Language**) and let Steam download it.
2. Open the Translator with that language as source. RPFM saves its vanilla texts and tells you when it's done.
3. Switch the game back to your language, and open the Translator again.

## Sharing through the Translation Hub

Once you've finished a translation, click **Submit to the Translation Hub** in the translator:

1. RPFM saves the translation, and shows a summary: the pack, the languages, how many lines are translated, and warnings about lines still pending or auto-translated and not reviewed. Fill in the authors there if you haven't, so you get credited, and click **Submit**.
2. The first time, RPFM asks you to sign in with GitHub. It opens GitHub in your browser and copies a code to your clipboard: paste it there and approve RPFM. You only need to do this once. RPFM keeps the sign-in in your system's keyring, and you can sign out from **Settings > GitHub**.
3. RPFM opens a pull request on the Translation Hub with your translation, through your own fork of it, which it creates the first time. If you submit the same translation again while its pull request is open, the pull request is updated instead.
4. Once the pull request is accepted, any Runcher user with **Enable Translations** turned on will get the translation automatically applied at launch — without altering the mod, and with `outdated` lines silently ignored. No more "I installed a translation pack but the mod was updated and now half the lines are wrong."

For step-by-step instructions including screenshots, see the [Translating a mod tutorial](../tutorials/translating-a-mod.md).
