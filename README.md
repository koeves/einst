# Einst

A native Apple Silicon rebuild of FXpansion **Geist** 1.1.1.2 (2012), the predecessor of Geist2.
It is built on [Dreist](../geist)'s Rust crates, because Geist v1 and Geist2 share most of their code:
93 of v1's 97 DSP classes and all 32 of its effects also exist in Geist2. See
[`docs/re/v1-vs-geist2.md`](docs/re/v1-vs-geist2.md).

Planned outputs: `Einst.vst3`, `Einst.clap`, `Einst.app`.

| Crate | Role |
|---|---|
| `einst-formats` | Geist v1 file reader, mapped onto Dreist's Geist2 document tree |

Status and next steps: [`docs/PROGRESS.md`](docs/PROGRESS.md).

`assets/geist1` holds FXpansion's original resources. Keep this repository private.
