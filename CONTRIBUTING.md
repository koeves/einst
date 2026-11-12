# Contributing

## Commits

- **Format**: [Conventional Commits](https://www.conventionalcommits.org/), `type(scope): summary`, with `!` after the type for a breaking change.
- **Types**: `feat`, `fix`, `perf`, `refactor`, `docs`, `test`, `build`, `ci`, `chore`, `style`, `revert`.
- **Checked**: by the `conventional` check; the changelog and release notes are generated from the commits.

## Branches and pull requests

- **Work**: on `feature/<name>`, one worktree per branch in `.claude/worktrees/<name>`.
- **Into `dev`**: a pull request, squash-merged; its title is the Conventional Commit.
- **Issues**: one per task; the pull request body says `Closes #N`.
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

- **CI**: `macos` (required) checks everything; `windows` and `linux-build` lint, test and bundle.
- **CodeQL**: runs on every pull request into `dev` and `main`.
- **Goldens**: stay identical unless a change means to alter them; say why in the commit.

## Releases

- **Release**: a pull request from `dev` into `main`, merged with a merge commit.
- **Automatic**: the merge computes the version from the commits, tags `vX.Y.Z`, builds macOS, Windows and Linux and publishes the GitHub release.
- **Follow-up**: a changelog and version pull request into `dev` merges itself once green.
- **Version**: fix → patch, feat → minor, breaking → major (minor while on 0.x); `Release-As: X.Y.Z` in the release pull request overrides it.
- **Releasable**: feat, fix, perf, refactor, revert, breaking and dependency commits; without one nothing is released.
- **Rebuild assets**: `gh workflow run release.yml --ref <tag> -f tag=<tag> -f upload=true`.

## dreist-commons

- **Submodule**: the shared crates are at `commons`; clone with `--recursive`.
- **Changes**: land in dreist-commons first, then bump the pointer here as `build: dreist-commons <sha or vX.Y.Z>`.
- **Releases**: `commons` must be at a dreist-commons release tag; release dreist-commons first.

## Builds

- **Signing**: `scripts/build.sh` signs ad hoc on macOS; `EINST_SIGN_IDENTITY` (Developer ID) signs with the hardened runtime, `EINST_NOTARY_PROFILE` also notarises.
- **Release builds**: unsigned on every system.
