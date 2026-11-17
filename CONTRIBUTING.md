# Contributing

## Commits

- **Format**: [Conventional Commits](https://www.conventionalcommits.org/), `type(scope): summary`, with `!` after the type for a breaking change.
- **Types**: `feat`, `fix`, `perf`, `refactor`, `docs`, `test`, `build`, `ci`, `chore`, `style`, `revert`.
- **Checked**: by the `conventional` check; versions and release notes are generated from the commits.

## Branches and pull requests

- **Work**: on `feature/<name>`, one worktree per branch in `.claude/worktrees/<name>`.
- **Into `dev`**: a pull request, squash-merged; its title is the Conventional Commit.
- **Drafts**: longer work opens as a draft pull request, marked ready once its testing is done; short work goes from a local branch straight to a ready pull request.
- **Issues**: for features, bug reports and work spanning several pull requests or repos; the pull request links it with `Closes #N`.
- **No issue**: small fixes, docs, chores and dependency bumps go straight to a pull request with a conventional title.
- **Board**: features and larger work for all three repos are tracked on [Dreist & Einst](https://github.com/users/koeves/projects/1).
- **Rules**: `dev` and `main` take changes only through pull requests with green checks.
- **Dependabot**: targets `dev`; patch, minor, Actions and submodule updates merge on their own when green.

## Checks

Run them before pushing:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --features einst-plugin/standalone -- -D warnings
cargo test --workspace --no-fail-fast
python3 scripts/uigen/uigen.py --check   # after changing scripts/uigen
```

- **CI**: pull requests into `dev` run `linux-build` (lint, test, bundle); the release pull request and the release also run `macos` (everything, effect fingerprints included) and `windows`.
- **Docs only**: a pull request that changes only Markdown skips the build and test jobs; a skipped job counts as passed.
- **CodeQL**: runs on every pull request into `dev` and `main`.
- **Goldens**: stay identical unless a change means to alter them; say why in the commit.

## Releases

- **Release**: when `dev` is ready, a pull request from `dev` into `main`, merged with a merge commit; its `release-version` check shows the version it releases.
- **Version**: from the commits since the last tag: fix → patch, feat → minor, breaking → major (minor while on 0.x); `Release-As: X.Y.Z` in the release pull request or a commit sets it.
- **Releasable**: feat, fix, perf, refactor, revert, breaking and dependency commits; `ci`, `docs`, `test`, `style` and other commits alone release nothing.
- **Automatic**: the merge tags `vX.Y.Z`, builds macOS, Windows and Linux and publishes the GitHub release, with the changes since the last tag as its notes.
- **Changes**: on the [Releases](https://github.com/koeves/einst/releases) page; there is no changelog file.
- **`dev` follows `main`**: after the merge, `dev` is fast-forwarded to `main` (the `DEV_SYNC_KEY` deploy key), or a sync pull request opens when `dev` has moved on.
- **Versions in the repository**: `Cargo.toml` keeps `0.0.0`; release builds take the tag's version, builds from `dev` the last tag and the commits since (`0.3.0+5`).
- **Rebuild assets**: `gh workflow run release.yml --ref <tag> -f tag=<tag> -f upload=true`.

## dreist-commons

- **Submodule**: the shared crates are at `commons`; clone with `--recursive`.
- **Changes**: land in dreist-commons first, then bump the pointer here as `build: dreist-commons <sha or vX.Y.Z>`.
- **Releases**: `commons` must be at a dreist-commons release tag; release dreist-commons first.

## Builds

- **Signing**: `scripts/build.sh` signs ad hoc on macOS; `EINST_SIGN_IDENTITY` (Developer ID) signs with the hardened runtime, `EINST_NOTARY_PROFILE` also notarises.
- **Release builds**: unsigned on every system.
