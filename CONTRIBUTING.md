# Contributing

Commit subjects follow [Conventional Commits](https://www.conventionalcommits.org/):
`type(scope): summary`, with type one of `feat`, `fix`, `perf`, `refactor`, `docs`, `test`,
`build`, `ci`, `chore`, `style` or `revert`, and `!` after the type for a breaking change. CI
checks every new commit (`.github/workflows/commits.yml`); the changelog is generated from them.

Releases: `git cliff --unreleased --tag vX.Y.Z --prepend CHANGELOG.md` ([git-cliff](https://git-cliff.org),
config in `cliff.toml`), edit the new section by hand if needed, bump the version in `Cargo.toml`,
commit as `chore(release): vX.Y.Z`, then tag `vX.Y.Z` and push the tag. The release workflow
publishes the tag's `CHANGELOG.md` section as the release notes.

Branches and CI: work happens on `feature/<name>` branches (parallel threads: one worktree each,
outside the checkout), pushed and merged into `dev` through a pull request with a merge commit
once its checks pass (CI, the commit-subject check and CodeQL). Dependabot targets `dev`: patch
and minor updates, GitHub Actions and submodule bumps merge on their own when the checks pass,
major updates wait for a review. A release is a pull request from `dev` into `main`, merged with
a merge commit; tagging that merge `vX.Y.Z` runs the release workflow. GitHub Actions run on pull
requests into `dev` and `main`, on release tags and by hand (`workflow_dispatch`), never on plain
branch pushes. Run the checks locally before pushing: `cargo fmt --all --check`,
`cargo clippy --workspace --all-targets --features einst-plugin/standalone -- -D warnings` and
`cargo test --workspace --no-fail-fast` (and `python3 scripts/uigen/uigen.py --check` after
changing `scripts/uigen`).

dreist-commons: the shared crates are the submodule at `commons` (clone with `--recursive`). To
change them, commit and push in `commons/` first, then commit the new pointer here as
`build: dreist-commons <short sha or vX.Y.Z>`; goldens must stay identical unless the change
means to alter them (say why in the commit).
