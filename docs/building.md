# Building from source

If you want to compile RPFM yourself — to contribute, to enable optional features, or to run the latest `develop` — these are the per-platform instructions.

## Windows

You'll need:

- [**Windows SDK**](https://developer.microsoft.com/en-US/windows/downloads/windows-sdk).
- **Visual Studio Community 2022** with the **MSVC** C++ build tools workload.
- **Rust 1.81** (or newer) with the MSVC toolchain.
- [**Craft**](https://community.kde.org/Craft) from KDE.

Once Craft is installed, install RPFM's KDE / Qt dependencies:

```bash
craft -i libs/qt6/qtimageformats
craft -i kimageformats
craft -i kwidgetsaddons
craft -i ktexteditor
craft -i kiconthemes
craft -i breeze-icons
```

### KDE designer plugins

By default Craft builds KDE Frameworks with `-DBUILD_DESIGNERPLUGIN=OFF`. RPFM uses `.ui` templates with KDE widgets (KLineEdit, KComboBox, KMessageWidget, …) that are loaded at runtime via `QUiLoader`. Without the designer plugins, `QUiLoader` cannot instantiate these widget types and RPFM crashes on startup.

Edit the Craft blueprint at `CraftRoot/etc/blueprints/locations/craft-blueprints-kde/kde/frameworks/frameworks.py` and change `BUILD_DESIGNERPLUGIN=OFF` to `BUILD_DESIGNERPLUGIN=ON`:

```python
self.subinfo.options.configure.args += [
    "-DBUILD_DESIGNERPLUGIN=ON",
    "-DBUILD_PYTHON_BINDINGS=OFF",
]
```

Then rebuild the relevant frameworks:

```bash
craft --fetch --unpack --configure --compile --install --qmerge kcompletion
craft --fetch --unpack --configure --compile --install --qmerge kwidgetsaddons
```

You should see `kcompletion6widgets.dll` and `kwidgetsaddons6widgets.dll` in `CraftRoot/plugins/designer/`.

### Building

Open Craft's terminal, move to RPFM's source folder and run:

```bash
# Debug build:
cargo build

# Run rpfm_ui in debug mode (auto-builds rpfm_server first):
cargo run --bin rpfm_ui

# Release build:
cargo build --release
```

You can make any editor inherit Craft's environment (and so be able to compile RPFM) by launching it from Craft's terminal.

## Linux

You'll need:

- **CMake**.
- **Rust 1.81** (or newer).
- **Qt6**.
- **KDE Frameworks 6**: KCompletion, KIconThemes, KTextEditor, KXmlGui, KWidgetsAddons.
- **xz**, **p7zip**.

Then from the repo root:

```bash
# Debug build:
cargo build

# Run rpfm_ui in debug mode:
cargo run --bin rpfm_ui

# Release build:
cargo build --release
```

## macOS

There's no maintained macOS build. The `qt_*` Qt6 bindings RPFM uses build on macOS in principle, but nobody is currently producing or testing macOS builds. Contributions welcome.

## Feature flags

Some features are gated behind cargo feature flags. Pass them with `--features`:

```bash
cargo run --bin rpfm_ui --features "enable_tools,support_uic"
```

Available flags (the up-to-date list lives in each crate's `Cargo.toml`):

| Flag                              | Default | What it enables |
|-----------------------------------|---------|-----------------|
| `enable_tools`                    | yes     | Compiles the integrated Tools menu (Translator, Faction Painter, Unit Editor). |
| `support_model_renderer`          | no      | 3D RigidModel preview. Requires the renderer library on the link path. |
| `support_uic`                     | no      | Propagates UIC parsing support through `rpfm_lib`. |
| `strict_subclasses_compilation`   | no      | Forces a compilation failure if `qt_rpfm_subclasses` fails to build. |
| `only_for_the_brave`              | no      | Exposes experimental features in the About dialog. |

## Building the manual

To build *this* manual locally:

```bash
# Install mdbook + the langtabs preprocessor RPFM uses:
cargo install --locked mdbook mdbook-langtabs

# Build the docs and open them in a browser:
mdbook build --open
```

To build the full site (landing page + manual + cargo doc API reference), use the orchestration script in the repo root:

```bash
./build_site.sh                # full build
./build_site.sh --skip-api     # manual + landing only
./build_site.sh --skip-manual  # landing + API only

# Preview locally:
python3 -m http.server -d out 8000
```

## Troubleshooting

- **Missing dll errors on Windows** — You're either not launching everything from Craft's terminal, or you're missing dependencies.

For anything else, the [issue tracker](https://github.com/Frodo45127/rpfm/issues) is the right place.
