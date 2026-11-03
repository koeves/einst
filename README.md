# Einst

A native Apple Silicon rebuild of FXpansion **Geist** 1.1.1.2 (2012), the predecessor of Geist2,
as a thin app on [dreist-commons](https://github.com/koeves/dreist-commons), the crates
it shares with Dreist (the Geist2 rebuild). Geist v1 and Geist2 share most of their code, so Einst
adds what v1 needs on top: its file reader, its own effect DSP and its behaviour where it differs.

Outputs: `Einst.vst3`, `Einst.clap`, `Einst.app` (Apple Silicon). Plugin IDs: CLAP
`rs.einst.einst`, VST3 class `EinstRsDrumSmplr`.

| Crate | Role |
|---|---|
| `einst` | Einst's app profile: names, theme, resources, the `.einst*` file family, v1's effect menu, v1 documents (`v1io`) and v1's effect nodes, installed with `einst::install()`; the engine and screen goldens under Einst's profile |
| `einst-formats` | Geist v1 file reader and converter onto the Geist2 document tree |
| `einst-dsp` | Geist v1's effect DSP where it differs from Geist2 (Gain, Vari-EQ, Envelope Shaper, and the shared effects whose v1 build differs) as `dreist_dsp` nodes |
| `einst-plugin` | nih-plug VST3 / CLAP plugin and standalone app |
| `xtask` | nih-plug bundler (`cargo xtask bundle einst-plugin`) |

## Build

dreist-commons is a git submodule at `commons` (clone with `--recursive`, or run
`git submodule update --init`), and Cargo uses its crates by path. To change commons, commit
and push inside `commons/`, then commit the new submodule revision here (`git add commons`).

    scripts/build.sh              # release: target/bundled/Einst.{app,vst3,clap}
    scripts/build.sh --debug
    scripts/build.sh --install    # also into ~/Library/Audio/Plug-Ins and ~/Applications
    cargo test --workspace
    cargo clippy --workspace --all-targets --features einst-plugin/standalone -- -D warnings
    cargo fmt --all --check

## Files

Einst opens Geist v1 files (`.geist`, `.geistkit`, `.geistpad`, `.geistengine`, `.geistpattern`,
`.geistbank`), Geist2 and Dreist files, and saves its own family (`.einst`, `.einstkit`, ...;
Geist2's XML). A v1 file is converted to the Geist2 document when the plugin loads its project
(the project path saved with the host session, or `EINST_PROJECT=<file>`) and when the editor
opens it: File › Open (single-object files land in engine 0), the browser, and drag and drop.
Samples resolve in `<file>.samples/` as in v1. Settings: `~/Library/Application Support/Einst`.

Resources: `assets/ui` (Einst's copy of the UI graphics: skin, Einst theme, fonts, cursors,
metronomes) and `assets/v1/System` (v1's factory effect presets, effect chains and templates).
`assets/v1` holds FXpansion's original resources; keep this repository private.

Status: [`docs/PROGRESS.md`](docs/PROGRESS.md). Reverse-engineering notes are in the private
`dreist-re` repository (`einst/`).

## Related projects

- [dreist-commons](https://github.com/koeves/dreist-commons) (public): the shared library
  (formats, DSP, engine, GUI, editor) that both apps are built on; a submodule here.
- [Dreist](https://github.com/koeves/dreist) (private): the Geist2 rebuild, an app on dreist-commons.
- [Einst](https://github.com/koeves/einst) (private): the Geist v1 rebuild, an app on dreist-commons.

The reverse-engineering notes behind all three are kept in a private repository.
