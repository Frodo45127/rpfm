# Video (CA_VP8)

`.ca_vp8` is a Creative Assembly–specific container around a VP8 video stream — used for movies, intros and similar. RPFM has full read/write support and can convert to and from the standard `.ivf` container.

<!-- IMAGE: CA_VP8 editor showing the stream metadata (width, height, frame count, framerate) and the convert / play buttons. -->

## What you can do

- **Inspect** stream metadata: width, height, frame count, framerate, codec version (`v0` or `v1`).
- **Convert to and from IVF** — can convert CA_VP8 files to and from IVF files, editable with ffmpeg.
- **Check video integrity** — verify the stream parses cleanly. Useful when debugging a video the game refuses to play.

## Editing a video

There's no in-app video editor. To re-encode a CA_VP8:

1. Convert the file to IVF (button in the editor toolbar).
2. Open the IVF in your video tool of choice and produce a new VP8 stream. FFmpeg works:
   ```bash
   ffmpeg -i input.mp4 -c:v vp8 -b:v 2M output.ivf
   ```
3. Convert back from IVF in the editor to bake the new stream into the `.ca_vp8`.
