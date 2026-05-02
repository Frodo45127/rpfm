# Images & DDS

The image editor is a **viewer** that handles most image formats CA uses, including DDS, PNG, JPG and the various atlas formats.

<!-- IMAGE: Image editor showing a DDS texture being previewed at 100 % zoom. -->

## What it does

- **Preview** the image — most DDS formats decode to PNG for display, including the BC1–BC7 compressed variants.
- **Zoom** with the mouse wheel; pan by dragging.

## What it doesn't do

There's no in-app pixel editor. To edit an image:

1. Extract it from the Pack (Pack tree → **Extract**).
2. Open it in your image editor of choice (GIMP, Photoshop, Krita…).
3. Save it back to disk.
4. Add it back to the Pack at the same path (Pack tree → **Add File…**), or use **Open with External Program** to wire your editor up so the round-trip happens automatically.

## Atlases

`.atlas` files are layout maps for sprite sheets — each entry references a region of an associated DDS. They open in their own simple table editor (under [Specialised editors](./specialised.md)) rather than the image viewer.
