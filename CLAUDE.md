# Einst (Rust rebuild of FXpansion Geist v1.1.1.2)

Goal: an Apple Silicon native rebuild of Geist v1 (standalone + CLAP/VST3), built on the crates
of **Dreist**, the Geist2 2.1.2.13 port at `../geist` (private repo `koeves/dreist`). Einst adds what v1
needs on top: the v1 file reader, the v1-only DSP, and v1's behaviour and layout where they differ.
Decision (user, 2026-11-02): separate Einst app with Dreist's crates as dependencies; no fork.

- `assets/v1` holds FXpansion's original v1 resources (effects, chains, templates, skins,
  manuals). Keep this repo private. Original binaries for RE are in `original/` (gitignored).
- RE notes: `docs/re/` (`v1-vs-geist2.md`, `formats-v1.md`). Tag claims [file]/[sym]/[decomp]/[runtime]
  and cite REA Evidence IDs, as in Dreist's `docs/re/README.md`.
- RE target: thin x86_64 engine `original/Geist-engine-x86_64` (sha256 `1751b602…8554`; the same
  binary backs v1's AU, VST and RTAS). For Geist2 comparisons use Dreist's target
  `/Library/Audio/Plug-Ins/VST/Geist2.vst/Contents/MacOS/Geist2`.
- Out of scope, as in Dreist: licensing (FX License Manager), updates/registration, RTAS,
  ManInTheMiddle (x86_64-only inter-plugin audio).
- REA (Ghidra 12.1.4 at `~/tools/ghidra_12.1.4_PUBLIC`, JDK 21) needs `GHIDRA_INSTALL_DIR`,
  `JAVA_HOME` and `REA_GHIDRA_STARTUP_TIMEOUT_MS=3600000` in the rea MCP env.

## Build

    export PATH=$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH
    cargo test --workspace
    cargo clippy --workspace --all-targets -- -D warnings
    cargo fmt --all --check
    scripts/build.sh          # target/bundled/Einst.{vst3,clap,app}

Dreist must be checked out next to this repo (`../geist`); Einst depends on its crates by path.
