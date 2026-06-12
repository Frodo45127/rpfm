#!/usr/bin/env python3
"""RPFM plugin example: wrap every Loc entry's text with an emoji.

RPFM extracts the files you selected to a temp folder and passes their paths as arguments.
DB and Loc tables arrive as TSV (tab-separated, quoting disabled), everything else as raw
binary. This script targets Loc TSVs and adds an emoji to the start and end of every `text`
value; RPFM then reads the result back into the Pack.

Drop this file in RPFM's config `scripts` folder, then right-click one or more loc files in
the contents tree and pick "Run Script". RPFM selects the interpreter from the `.py`
extension, so `python` must be on PATH.
"""

import sys

# Emoji written as a code point so this file carries no literal emoji glyph.
EMOJI = "\U0001F525"  # fire
PREFIX = EMOJI + " "
SUFFIX = " " + EMOJI

# Table types RPFM writes in the metadata row of a Loc TSV.
LOC_TYPES = ("Loc", "Loc PackedFile")


def process_tsv(path):
    with open(path, "r", encoding="utf-8", newline="") as handle:
        # splitlines() drops the line terminators and copes with both '\n' and '\r\n'.
        lines = handle.read().splitlines()

    # A valid table TSV always has a column-names row and a metadata row.
    if len(lines) < 2:
        return

    metadata = lines[1].lstrip("#").split(";")
    if not metadata or metadata[0] not in LOC_TYPES:
        print("Skipping non-Loc TSV: " + path)
        return

    header = lines[0].split("\t")
    if "text" not in header:
        return
    text_index = header.index("text")

    # Lines 0 (column names) and 1 (metadata) are kept verbatim; the rest are data rows.
    updated = 0
    for i in range(2, len(lines)):
        if not lines[i]:
            continue
        cells = lines[i].split("\t")
        if text_index < len(cells):
            cells[text_index] = PREFIX + cells[text_index] + SUFFIX
            lines[i] = "\t".join(cells)
            updated += 1

    # RPFM writes '\r\n'-terminated rows (including the last one), so we match that.
    with open(path, "w", encoding="utf-8", newline="") as handle:
        handle.write("\r\n".join(lines) + "\r\n")

    print("Updated {} entries in {}".format(updated, path))


def main():
    for path in sys.argv[1:]:
        if path.lower().endswith(".tsv"):
            process_tsv(path)


if __name__ == "__main__":
    main()
