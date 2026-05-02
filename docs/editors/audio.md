# Audio

CA's audio pipeline is built on Wwise and ships as several related artefacts inside Packs:

- **Sound banks** (`.bnk`) — Wwise-format containers with the audio assets.
- **Sound bank databases** — table-shaped indices that match bank IDs to game-side names.
- **Sound events** — DB-shaped tables the rest of the game references when something needs to play.

In **RPFM today** the only dedicated UI is a play/stop **media player** for individual audio files (`.wav`, `.ogg`, …).

<!-- IMAGE: Audio editor showing a play / stop button pair for a single audio file. -->

## What you can do

- **Play / stop** any audio file the editor recognises (the audio editor view is just two toolbar buttons).

## What you can't (yet)

- Author or repackage a `.bnk` from raw audio inside RPFM.
- Inspect a `.bnk`'s internal event list, listed assets, or other internal metadata.

For new audio content, use Asset Editor.
