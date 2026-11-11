# Contributing

Commit subjects follow [Conventional Commits](https://www.conventionalcommits.org/):
`type(scope): summary`, with type one of `feat`, `fix`, `perf`, `refactor`, `docs`, `test`,
`build`, `ci`, `chore`, `style` or `revert`, and `!` after the type for a breaking change. CI
checks every new commit (`.github/workflows/commits.yml`); the changelog is generated from them.

Releases: merging the release pull request (`dev` -> `main`, merge commit) releases; there is no
manual tagging, bumping or changelog commit. `.github/workflows/release-on-main.yml` computes the
version from the commits since the last `v*` tag (`git cliff --bumped-version`, rules in
`cliff.toml`: fix and the rest -> patch, feat -> minor, breaking -> major, and breaking -> minor
while on 0.x), checks that `commons` points at a dreist-commons release tag, tags the merge commit
`vX.Y.Z` and runs `release.yml`: the commit check, tests and the macOS, Windows and Linux
builds (VST3, CLAP, app zips; tarballs on Linux) with the version written into `Cargo.toml` in its own checkout (plugins, app and About show
it), and the GitHub release with notes generated from the commits. It then opens the follow-up
pull request `docs(release): changelog and version for vX.Y.Z` into `dev` (CHANGELOG.md, the
`Cargo.toml` / `Cargo.lock` version), which merges itself once its checks pass. Only feat, fix,
perf, refactor, revert, breaking and dependency (`build(deps)`, `chore(deps)`, `build:
dreist-commons ...`) commits are releasable: without one nothing is released and the run says so.
A line `Release-As: X.Y.Z` in the release pull request's body (or in a commit message since the
last tag) sets the version instead, and releases even without a releasable commit. Release
dreist-commons first and point `commons` at its tag (a pull request into `dev`) when Einst needs
the change. `release.yml` also runs on a tag pushed by hand; to rebuild a release's assets: `gh
workflow run release.yml --ref <tag> -f tag=<tag> -f upload=true`.

Security alerts: check the open code scanning and Dependabot alerts when starting and when
finishing work on the repository (`gh api repos/koeves/einst/code-scanning/alerts?state=open`,
`gh api repos/koeves/einst/dependabot/alerts?state=open`). Fix real findings through an issue and a
pull request; dismiss false positives with the reason in the dismissal comment.

Branches and CI: work happens on `feature/<name>` branches (parallel threads: one worktree each,
outside the checkout), pushed and merged into `dev` through a pull request by squash only: the pull request title is
a Conventional Commit subject and becomes the commit (checked by `conventional`). Merge once its
checks pass (CI, `conventional`, CodeQL). Dependabot targets `dev`: patch
and minor updates, GitHub Actions and submodule bumps merge on their own when the checks pass,
major updates wait for a review. A release is a pull request from `dev` into `main`, merged with a
merge commit only, never squashed (it keeps `dev`'s commits, which give the version and the
changelog; see Releases). Rules: `dev` and `main` require a pull request (no approvals) and the
checks; `dev` allows squash only, `main` merge commits only. GitHub Actions run on pull requests into `dev` and `main`, on pushes
to `main` (the release), on release tags and by hand (`workflow_dispatch`, also how the follow-up
pull request gets its checks), never on other branch pushes. Run the checks locally before
pushing: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --features
einst-plugin/standalone -- -D warnings` and `cargo test --workspace --no-fail-fast` (and `python3
scripts/uigen/uigen.py --check` after changing `scripts/uigen`).

Platforms: CI's `macos` job (required) checks everything on macOS; `windows` and `linux-build`
(not required) lint, test and bundle with `scripts/build-windows.sh` (Git Bash) and
`scripts/build-linux.sh` (the system packages in the README; tests under `xvfb-run`). Windows and
Linux binaries are unsigned.

dreist-commons: the shared crates are the submodule at `commons` (clone with `--recursive`). To
change them, commit and push in `commons/` first, then commit the new pointer here as `build:
dreist-commons <short sha or vX.Y.Z>`; goldens must stay identical unless the change means to
alter them (say why in the commit). A release needs `commons` at a dreist-commons release tag (the
release run checks it): release dreist-commons first, then bump the pointer to its tag (`build:
dreist-commons vX.Y.Z`, or Dependabot's pull request).
