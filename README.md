# Einst

- A drum sampler, step sequencer and performance instrument: 8 engines of pads and layers,
  patterns, scenes, song mode, slicing and effects.
- Rebuilt in Rust from the reverse-engineered behaviour of FXpansion **Geist** 1.1.1.2 (2012),
  the predecessor of Geist2, with v1's look, effects and sound.
- Opens Geist v1 projects, kits, pads, engines and patterns (and Geist2 and Dreist files);
  saves its own `.einst` family (`.einst`, `.einstkit`, ...).
- VST3, CLAP and a standalone app for macOS (Apple Silicon), Windows (x64) and Linux (x86_64,
  X11 / XWayland); minimums under [Installation](#installation).
- Settings: `~/Library/Application Support/Einst` (Windows: `%APPDATA%\Einst`, Linux:
  `~/.config/Einst`, or `$XDG_CONFIG_HOME/Einst`).
- Plugin IDs: CLAP `rs.einst.einst`, VST3 class `EinstRsDrumSmplr`.

| | |
|---|---|
| ![Pattern editor](docs/screens/readme/einst-pattern.png) | ![Pad / Layers](docs/screens/readme/einst-pad.png) |
| ![Song mode](docs/screens/readme/einst-song.png) | ![Engine mixer](docs/screens/readme/einst-engine.png) |
| ![Scenes](docs/screens/readme/einst-scene.png) | ![Global mixer](docs/screens/readme/einst-global.png) |

## Installation

Download the archives for your system from the
[releases page](https://github.com/koeves/einst/releases) (latest:
[v0.1.0](https://github.com/koeves/einst/releases/tag/v0.1.0), macOS only; Windows and Linux
builds come with the next release).

Supported systems:

- macOS 11 or newer on Apple Silicon.
- Windows 10 or newer, x64.
- Linux x86_64 with glibc 2.39 or newer (Ubuntu 24.04, Fedora 40 or newer), X11 or XWayland.

Release downloads, per system (`<tag>` is the release, say `v0.1.0`):

| System | VST3 | CLAP | App |
|---|---|---|---|
| macOS | `Einst-<tag>-macOS-VST3.zip` | `Einst-<tag>-macOS-CLAP.zip` | `Einst-<tag>-macOS-App.zip` |
| Windows | `Einst-<tag>-Windows-VST3.zip` | `Einst-<tag>-Windows-CLAP.zip` | `Einst-<tag>-Windows-App.zip` |
| Linux | `Einst-<tag>-Linux-VST3.tar.gz` | `Einst-<tag>-Linux-CLAP.tar.gz` | `Einst-<tag>-Linux-App.tar.gz` |

**macOS**

- VST3: unzip and move `Einst.vst3` to `~/Library/Audio/Plug-Ins/VST3/`.
- CLAP: move `Einst.clap` to `~/Library/Audio/Plug-Ins/CLAP/`.
- App: move `Einst.app` to `/Applications` (or `~/Applications`).

The builds are not signed or notarised. Open the app the first time with right-click (or
Control-click) › Open › Open. If a host refuses to load the plugins, clear the quarantine flag:

```bash
xattr -dr com.apple.quarantine ~/Library/Audio/Plug-Ins/VST3/Einst.vst3 ~/Library/Audio/Plug-Ins/CLAP/Einst.clap
```

**Windows**

- VST3: unzip and move the `Einst.vst3` folder to `C:\Program Files\Common Files\VST3\`.
- CLAP: move the `Einst` folder (holding `Einst.clap` and its `Resources`) to
  `C:\Program Files\Common Files\CLAP\`.
- App: unzip the `Einst` folder anywhere and run `Einst.exe`. Keep the `Resources` folder next
  to it. SmartScreen may warn on first launch, because the build is unsigned: More info › Run
  anyway.

**Linux** (x86_64, glibc 2.39+; X11, Wayland desktops run it through XWayland)

- VST3: unpack and move `Einst.vst3` to `~/.vst3/` (or `/usr/lib/vst3/`).
- CLAP: unpack and move the `Einst` folder (holding `Einst.clap` and its `Resources`) to
  `~/.clap/` (or `/usr/lib/clap/`).
- App: unpack the `Einst` folder anywhere (say `~/.local/share/Einst`) and run `Einst`; keep
  `Resources` next to it. A symlink to it (`ln -s ~/.local/share/Einst/Einst ~/.local/bin/einst`)
  works, and `einst.desktop` / `einst.png` are a desktop entry and icon for
  `~/.local/share/applications` and `~/.local/share/icons/hicolor/256x256/apps`.
- Runtime needs: ALSA (`libasound2`), X11 / XCB (`libx11-6`, `libx11-xcb1`, `libxcb1`,
  `libxcursor1`) and OpenGL (`libgl1`), present on desktop systems. The standalone app plays
  through ALSA (with PipeWire or PulseAudio, through their ALSA plugin); file dialogs use the XDG
  desktop portal (`xdg-desktop-portal`, or `zenity` without one).

**Known limitations**

- Windows and Linux: REX files (`.rx2`, `.rex`) do not load, and samples cannot be dragged out
  of Einst into a host or the file manager (both macOS only).
- Linux: X11 only; Wayland desktops run Einst through XWayland.
- The builds are unsigned on every system (see above for the first launch on macOS and Windows).

**Content**

- Einst does not include sample content. Geist v1 and Geist2 kits, presets and expansions load
  as they are.
- v1's factory effect presets, effect chains and pattern and swing templates are bundled.
- A project's samples are found next to it in `<project>.samples/`, as in v1.

## Build

Needs a Rust toolchain.

```bash
scripts/build.sh            # macOS: target/bundled/Einst.{app,vst3,clap}
scripts/build.sh --install  # also copies them to ~/Library and /Applications
scripts/build-windows.sh    # Windows (Git Bash): target/bundled/Einst.vst3, Einst-CLAP/, Einst-App/
scripts/build-linux.sh      # Linux: target/bundled/Einst.vst3, Einst-CLAP/, Einst-App/
scripts/build-linux.sh --install  # also into ~/.vst3, ~/.clap/Einst, ~/.local/share/Einst (+ ~/.local/bin/einst)
cargo test --workspace
```

On Linux the build needs (Debian / Ubuntu names; Fedora: `alsa-lib-devel libX11-devel
libxcb-devel libXcursor-devel mesa-libGL-devel`):

```bash
sudo apt-get install pkg-config libasound2-dev libx11-dev libx11-xcb-dev libxcb1-dev libxcursor-dev libgl1-mesa-dev
```

Merging a release pull request (`dev` -> `main`) releases: `.github/workflows/release-on-main.yml`
takes the version from the commits, tags the merge `vX.Y.Z`, tests and builds on macOS, Windows and
Linux, publishes the GitHub release with the zips (Linux: tarballs) and opens the changelog pull request into `dev`
([CONTRIBUTING.md](CONTRIBUTING.md)).

## Layout

Einst is an app on [dreist-commons](https://github.com/koeves/dreist-commons), the crates it
shares with Dreist (the Geist2 rebuild). This repository adds what v1 does differently:

| Crate | Role |
|---|---|
| `einst` | App profile: names, theme, resources, file family, v1's effect menu and effect nodes |
| `einst-editor` | Einst's editor: v1's panels, dialogs, menus and settings |
| `einst-formats` | Geist v1 file reader, converted onto the shared document |
| `einst-dsp` | v1's own effects (Gain, Vari-EQ, Envelope Shaper) and v1's versions of the shared ones |
| `einst-plugin` | nih-plug VST3 / CLAP / standalone |
| `xtask` | nih-plug bundler |

`assets/ui-v1` is the v1-layout UI set, generated by `scripts/uigen/uigen.py` from Einst's own
layout source (`scripts/uigen/skins`) and theme; `assets/factory` holds v1's factory presets
and templates.

## Contributing

- Commits follow Conventional Commits (`feat(scope): ...`); the changelog and release notes
  are generated from them ([CONTRIBUTING.md](CONTRIBUTING.md)).
- Checks: `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets --features einst-plugin/standalone -- -D warnings`,
  `cargo test --workspace`.
- dreist-commons is a submodule at `commons`: clone with `--recursive`.

## Related projects

- [dreist-commons](https://github.com/koeves/dreist-commons): the shared library (formats, DSP,
  engine, GUI toolkit) that both apps are built on.
- [Dreist](https://github.com/koeves/dreist): the Geist2 rebuild, an app on dreist-commons.
- [Einst](https://github.com/koeves/einst): the Geist v1 rebuild, an app on dreist-commons.

## Licence

The crates are MIT OR Apache-2.0. `einst-plugin` and the binaries built from it are
GPL-3.0-or-later. See [`LICENSE`](LICENSE) and [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).
Einst is not affiliated with or endorsed by FXpansion.

## AI usage

Built with assistance of Claude using REA.

Tested by hand and by ear.
