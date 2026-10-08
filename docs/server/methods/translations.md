<!-- Generated from the API types by `rpfm_ipc/src/api/method_docs.rs`. Don't edit it by hand. -->

# Translations

Methods about the translations of the open packs.

## `translations.pack`

Returns the whole translation of an open pack to a language, as the translator works with it.

Meant for clients editing whole translations. Others should use `translations.list`.

**Params** (`GetPackTranslation`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `source_language` | `string` | yes | Language code of the source texts, like `en`. |
| `language` | `string` | yes | Language code of the translation, like `es`. |

**Result** (`PackTranslation`)

One of RPFM's own types, without a schema. See its docs in `rpfm_lib` or `rpfm_extensions`.

## `translations.submit`

Submits the saved translation of a pack to the Translation Hub, opening or updating a pull request.

**Params** (`SubmitTranslation`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack_name` | `string` | yes | Name of the pack the translation is for. |
| `source_language` | `string` | yes | Language code of the source texts, like `en`. |
| `language` | `string` | yes | Language code of the translation, like `es`. |

**Result** (`TranslationSubmission`)

One of:

- `{ "status": "submitted", "created": …, "url": … }`: The translation was submitted.
    - `created` (`boolean`, yes): If a new pull request was opened. `false` means an open one was updated.
    - `url` (`string`, yes): Web page of the pull request.
- `{ "status": "sign_in_required" }`: The user has to sign in to GitHub first, because they never did or GitHub rejected their sign-in.

## `translations.list`

Returns the translation of the texts of an open pack to a language.

Texts already translated in the vanilla game, or in previous translations of the pack, come translated.

**Params** (`ListTranslations`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pack` | `string` | yes | Key of the pack. |
| `source_language` | `string` | no, default `"EN"` | Language the texts of the pack are written in, like `EN`. |
| `language` | `string` | yes | Language to translate the texts to, like `ES` or `DE`. |
| `filter` | [`TranslationFilter`](#type-translationfilter) | no, default `"all"` | Which entries to return. |
| `offset` | `integer` | no, default `0` | Amount of matching entries to skip. |
| `limit` | `integer` or `null` | no | Maximum amount of entries to return. Defaults to 200. |

**Result** (`Translations`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `entries` | array of [`TranslationEntry`](#type-translationentry) | yes | The entries in the page, sorted by key. |
| `total` | `integer` | yes | Amount of entries matching the request, in all pages. |

## `translations.generate_vanilla`

Generates the vanilla texts of a language from the game's locale packs,
so translations to and from it can reuse them.

**Params** (`GenerateVanillaTexts`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `language` | `string` | yes | Language of the texts to generate, like `EN`. |

**Result** (`VanillaTextsAvailable`)

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `available` | `boolean` | yes | If they're available, generated now or before. |

## Types

### Type `TranslationEntry`

The translation of a text.

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `key` | `string` | yes | Loc key of the text. |
| `source` | `string` | yes | Text in the source language. |
| `translation` | `string` | yes | Text in the target language. Empty if it's not translated yet. |
| `needs_retranslation` | `boolean` | yes | If the source text changed since it was translated. |
| `automatic` | `boolean` | yes | If the translation was made automatically, and still needs a review. |
| `removed` | `boolean` | yes | If the text no longer exists in the pack. |

### Type `TranslationFilter`

Which translation entries to return.

One of:

- `"all"`: All of them.
- `"untranslated"`: Entries without a translation.
- `"needs_review"`: Entries whose source text changed since they were translated, or that were translated automatically.
