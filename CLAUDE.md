# Einst

Rebuild of the original v1 instrument as a thin app on **dreist-commons** (a git submodule at
`commons`, used by path; see `CONTRIBUTING.md` for changing it). Einst plugs in through
dreist-commons' hooks: `crates/einst` holds the app profile (`dreist_formats::profile`) and installs v1's effect nodes
(`dreist_dsp::fx::install_factory`). Shared behaviour belongs in dreist-commons, v1 specifics here.

- Outside README files: no original product, vendor or class names in code, comments or docs. Cite
  reverse-engineering evidence by Evidence ID (`ev_…`) or tag (`[re:xxxxxx]`); the tag map
  (`einst/evidence-tags.tsv`), the RE ledger (`einst/docs/re/`), the progress log up to the move,
  and the RE scripts (`einst/re/scripts/`, including `rea_drive.py`) are in the private
  `dreist-re` repository (`../dreist-re`). Original binaries for RE stay in `original/`
  (gitignored).
- Goldens (`crates/einst/tests/golden`) must stay identical unless a change is meant to alter
  them; explain every change. Porting bugs found in dreist-commons get their own commit there.
- Out of scope: licensing / copy protection, updates and registration, RTAS, inter-plugin audio.
- Keep the repository private. Git (Conventional Commits, changelog, releases, branches,
  submodule bumps): `CONTRIBUTING.md`; agent workflow notes: dreist-re `agents.md`.

## Build

    export PATH=$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH
    cargo test --workspace
    cargo clippy --workspace --all-targets --features einst-plugin/standalone -- -D warnings
    cargo fmt --all --check
    scripts/build.sh          # target/bundled/Einst.{vst3,clap,app}
    python3 tools/plugincheck.py target/bundled/Einst.clap/Contents/MacOS/Einst target/bundled/Einst.vst3/Contents/MacOS/Einst
    tools/editorhost.sh target/bundled/Einst.clap target/bundled/Einst.vst3 ../dreist/target/bundled/Dreist.clap

Nothing may come to the front while the desktop is in use: tests and harnesses set
`DREIST_BACKGROUND_GUI=1` (commons `vendor/baseview`), the standalone app is not run for
tests, pluginval runs with `--skip-gui-tests`, and apps start with `open -g`.

Branches: work on local feature branches or local `dev` (parallel threads: one worktree each,
outside the checkout); push nothing while working. At a release, clean up `dev`'s commits, push `dev`,
merge into `main` (`--no-ff`) and release.

After any code change here or in commons, rebuild and reinstall the VST3 (`scripts/build.sh`, then
`target/bundled/Einst.vst3` into `~/Library/Audio/Plug-Ins/VST3`) and say which commit is installed.
