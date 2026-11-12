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
  (gitignored); v1's install files are in dreist-re `einst/v1` (tests that need them read
  `EINST_V1_ASSETS`, default `../dreist-re/einst/v1`, and skip without). This repository keeps
  only v1's factory presets and templates (`assets/factory`).
- UI: `scripts/uigen/skins` is Einst's own layout source for the v1-layout set; edit it (or
  theme.py / authored.py), then `python3 scripts/uigen/uigen.py` writes `assets/ui-v1`
  (`--check` fails when it is stale).
- Goldens (`crates/einst/tests/golden`) must stay identical unless a change is meant to alter
  them; explain every change. Porting bugs found in dreist-commons get their own commit there.
- Out of scope: licensing / copy protection, updates and registration, RTAS, inter-plugin audio.
- Git (Conventional Commits, changelog, releases, branches,
  submodule bumps): `CONTRIBUTING.md`; agent workflow notes: dreist-re `agents.md`.
- Security alerts: check the open code scanning and Dependabot alerts at the start and end of any
  work here (`gh api repos/koeves/einst/code-scanning/alerts?state=open`, same for
  `dependabot/alerts`); fix real findings through an issue and a PR, dismiss false positives
  with the reason.

## Docs

- Human docs (README, CONTRIBUTING, other `.md` for readers) follow the fzf README style: a centred title with release, build, CodeQL, licence and platform badges under it.
- Short bullets of one sentence, two at most; no walls of text, and detail goes in its own section or file.
- Describe the project as it is: no status updates, no "now", "previously", "since" or "next release" wording, and nothing the reader doesn't need.
- This file is the agent instructions for every agent; `CLAUDE.md`, `GEMINI.md` and `.github/copilot-instructions.md` only point here, so edit this file.
- Agent notes stay in this file or in dreist-re; CHANGELOG.md and THIRD_PARTY_NOTICES.md are left to their generators.

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

Branches: `feature/<name>` (one worktree each, outside the checkout) -> pull request into
`dev` (squash only; the PR title is the Conventional Commit; checks green) -> release pull request `dev` -> `main` (merge commit). The
merge releases: version from the commits (`Release-As: X.Y.Z` overrides), tag, build, GitHub
release and a follow-up changelog / version pull request into `dev` happen on their own; never
tag or bump by hand. `commons` must be at a dreist-commons release tag (CONTRIBUTING.md).

After any code change here or in commons, rebuild and reinstall the VST3 (`scripts/build.sh`, then
`target/bundled/Einst.vst3` into `~/Library/Audio/Plug-Ins/VST3`) and say which commit is installed.
