#!/usr/bin/env bash
# Build the full RPFM site locally:
#   out/                landing page (from website/)
#   out/manual/         mdbook output (from docs/ + book.toml)
#   out/api/            cargo doc --no-deps for the portable libraries
#
# Usage: ./build_site.sh [--skip-api] [--skip-manual] [--out DIR]
#
# Requires:
#   - mdbook            (cargo install mdbook)
#   - mdbook-langtabs   (cargo install mdbook-langtabs)
#   - cargo, rustdoc    (default toolchain)

set -euo pipefail

OUT="out"
SKIP_API=0
SKIP_MANUAL=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --skip-api)    SKIP_API=1; shift ;;
        --skip-manual) SKIP_MANUAL=1; shift ;;
        --out)         OUT="$2"; shift 2 ;;
        -h|--help)
            sed -n '2,12p' "$0"
            exit 0
            ;;
        *) echo "unknown flag: $1" >&2; exit 2 ;;
    esac
done

ROOT="$(cd "$(dirname "$0")" && pwd)"
OUT_ABS="$ROOT/$OUT"

# Crates that ship API docs, in dependency order (deps first). Building one at a
# time in this order ensures each crate's deps are already in the target dir when
# rustdoc runs, so cross-crate type links resolve to their definitions instead of
# rendering as plain text.
#
# Qt-bound crates (rpfm_ui, rpfm_ui_common) are skipped — they need Qt6 + KDE
# Frameworks 6 to compile, which makes the runner unhappy.
API_CRATES=(rpfm_lib rpfm_telemetry rpfm_ipc rpfm_extensions rpfm_server)

echo ">> cleaning $OUT/"
rm -rf "$OUT_ABS"
mkdir -p "$OUT_ABS"

echo ">> copying landing page from website/"
cp -R "$ROOT/website/." "$OUT_ABS/"

if [[ $SKIP_MANUAL -eq 0 ]]; then
    echo ">> building mdbook → $OUT/manual/"
    if ! command -v mdbook >/dev/null 2>&1; then
        echo "!! mdbook not installed; run: cargo install mdbook mdbook-langtabs" >&2
        exit 1
    fi
    mdbook build "$ROOT" --dest-dir "$OUT_ABS/manual"
else
    echo ">> skipping manual"
fi

if [[ $SKIP_API -eq 0 ]]; then
    echo ">> building cargo doc for: ${API_CRATES[*]}"
    DOC_TARGET="$ROOT/target_doc"
    # Document one crate at a time, in dep order. `--no-deps -p X` doesn't always
    # serialise build order across multiple `-p` flags, which leaves cross-crate
    # type references unlinked when the dep's docs aren't on disk yet. Looping
    # ensures each previous crate's docs are already in $DOC_TARGET/doc/ when
    # rustdoc runs for the next one.
    for c in "${API_CRATES[@]}"; do
        echo ">>   $c"
        (cd "$ROOT" && cargo doc --no-deps --target-dir "$DOC_TARGET" -p "$c")
    done

    echo ">> copying cargo doc → $OUT/api/"
    mkdir -p "$OUT_ABS/api"
    cp -R "$DOC_TARGET/doc/." "$OUT_ABS/api/"

    # cargo doc with --no-deps -p X -p Y doesn't generate a root index.html,
    # so navigating to /api/ would 404. Generate a tiny landing page.
    {
        printf '<!doctype html><html lang="en"><head><meta charset="utf-8">'
        printf '<title>RPFM &mdash; Library API</title>'
        printf '<link rel="icon" type="image/png" href="../assets/logo.png">'
        printf '<link rel="stylesheet" href="../style.css"></head><body>'
        printf '<header class="site-header"><a class="brand" href="../">'
        printf '<img src="../assets/logo.png" alt="" class="brand-mark">'
        printf '<span class="brand-name">Rusted PackFile Manager</span></a>'
        printf '<nav class="primary-nav"><a href="../">Home</a><a href="../manual/">Manual</a><a href="https://github.com/Frodo45127/rpfm">GitHub</a></nav></header>'
        printf '<main><section class="section"><h1 class="section-title">Library API</h1>'
        printf '<p class="section-lede">Generated with <code>cargo doc --no-deps</code>. Pick a crate:</p>'
        printf '<div class="get-started">'
        for c in "${API_CRATES[@]}"; do
            desc="$(grep -m1 '^description' "$ROOT/$c/Cargo.toml" 2>/dev/null | sed -E 's/^description\s*=\s*"(.*)"/\1/')"
            [[ -z "$desc" ]] && desc="No description provided in <code>${c}/Cargo.toml</code>."
            printf '<article class="gs-card"><h3>%s</h3>' "$c"
            printf '<p>%s</p>' "$desc"
            printf '<a class="cta cta-small" href="./%s/index.html">Open</a></article>' "$c"
        done
        printf '</div></section></main></body></html>'
    } > "$OUT_ABS/api/index.html"
else
    echo ">> skipping api docs"
fi

echo
echo ">> done."
echo "   Site root:   $OUT_ABS/"
[[ $SKIP_MANUAL -eq 0 ]] && echo "   Manual:      $OUT_ABS/manual/"
[[ $SKIP_API    -eq 0 ]] && echo "   API docs:    $OUT_ABS/api/"
echo
echo "Preview locally with any static file server, e.g.:"
echo "   python3 -m http.server -d $OUT 8000"
