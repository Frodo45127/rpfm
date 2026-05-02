# Installation

RPFM ships pre-built for Windows and Linux. macOS isn't supported yet — there's no maintained build.

## Windows

1. Download the latest `rpfm-vX.Y.Z-x86_64-pc-windows-msvc.zip` from the [releases page](https://github.com/Frodo45127/rpfm/releases).
2. Extract the archive anywhere — your Documents folder, a tools folder on a secondary drive, anywhere you like. RPFM is portable; it doesn't need an installer.
3. Run `rpfm_ui.exe`.

That's it. The first launch will take you to [first-time configuration](./first-time-config.md).

> **Heads-up:** Don't extract the zip into a path that requires admin rights (e.g. `Program Files`). RPFM writes certain update-related files relative to where its files are, and a write-protected install path causes confusing errors.

## Linux

### Arch Linux and derivatives

The recommended install is the `rpfm-bin` package on the [AUR](https://aur.archlinux.org/packages/rpfm-bin):

```bash
# With your favourite AUR helper
paru -S rpfm-bin
# or yay -S rpfm-bin
```

There's also `rpfm-git` if you want to build from the latest `develop` branch yourself.

### Other distributions (Flatpak)

A Flatpak is the easiest way to run RPFM on any distro that supports Flatpak:

<!-- IMAGE: Optional — Flatpak install command in a terminal. Probably skip. -->

The Flatpak bundles Qt6 and the KDE Frameworks RPFM needs, so you don't have to install them yourself. Refer to the project's [releases page](https://github.com/Frodo45127/rpfm/releases) for the current Flatpak download.

### Building from source

If your distro doesn't have a maintained package and the Flatpak doesn't fit, see [Building from source](../building.md).

## macOS

There's no maintained macOS build. The `qt_*` Qt6 bindings RPFM uses build on macOS in principle, but nobody is currently producing or testing macOS releases. If you want to take that on, contributions are welcome.

## Updating

By default RPFM checks for updates on launch and shows a dialog when one is available. The update flow downloads the new version, replaces the binaries and restarts. Beta and stable channels are configurable from **Preferences → Updates**.

> On Linux, in-app updates are disabled — your distribution's package manager (or Flatpak) is the one in charge of updates. Update through the same channel you used to install.

## Verifying it works

After launching for the first time you should see the **welcome page**, with quick links to the manual, recent Packs, and your update status. If you got that far, the install is good. Move on to [first-time configuration](./first-time-config.md).
