# Einst

A native Apple Silicon rebuild of FXpansion **Geist** 1.1.1.2 (2012), the predecessor of Geist2.
It is built on [Dreist](../geist)'s Rust crates, because Geist v1 and Geist2 share most of their code:
93 of v1's 97 DSP classes and all 32 of its effects also exist in Geist2. See
[`docs/re/v1-vs-geist2.md`](docs/re/v1-vs-geist2.md).

Outputs: `Einst.vst3`, `Einst.clap`, `Einst.app` (Apple Silicon).

| Crate | Role |
|---|---|
| `einst-formats` | Geist v1 file reader and converter onto Dreist's Geist2 document tree |
| `einst-dsp` | Geist v1's own effect DSP (Gain, Vari-EQ, Envelope Shaper) as `geist_dsp` nodes |
| `einst-engine` | Dreist's `geist-engine` copied, building those nodes (used under the name `geist-engine`) |
| `einst-editor` | the editor: a copy of Dreist's `geist-editor` with Einst's names, settings folder, resources (v1 factory presets and templates) and v1 file opening (source commit in `docs/PROGRESS.md`) |
| `einst-plugin` | nih-plug VST3 / CLAP plugin and standalone app: a copy of Dreist's `geist-plugin` shell with Einst's names and IDs (CLAP `rs.einst.einst`, VST3 class `EinstRsDrumSmplr`), opening Geist v1 files |
| `xtask` | nih-plug bundler (`cargo xtask bundle einst-plugin`) |

## Build

Dreist must be checked out next to this repository (`../geist`): Einst builds on its crates by path,
uses its vendored `baseview` / `signalsmith-stretch` patches, and bundles its resources.

    scripts/build.sh              # release: target/bundled/Einst.{app,vst3,clap}
    scripts/build.sh --debug
    scripts/build.sh --install    # also into ~/Library/Audio/Plug-Ins and ~/Applications
    cargo test --workspace
    cargo clippy --workspace --all-targets -- -D warnings
    cargo fmt --all --check

## Opening Geist v1 files

A Geist v1 `.geist`, `.geistkit`, `.geistpad`, `.geistengine`, `.geistpattern` or `.geistbank` is
converted to Geist2's document (`einst_formats::v1::open_as_project`) when the plugin loads its
project (the project path saved with the host session, or `EINST_PROJECT=<file>`) and when the
editor opens it: File › Open (any of these extensions; single-object files land in engine 0), the
browser, and drag and drop of projects, kits and engines. Samples resolve in `<file>.samples/` as
in v1.

Status and next steps: [`docs/PROGRESS.md`](docs/PROGRESS.md).

`assets/geist1` holds FXpansion's original resources. Keep this repository private.
