# Contributing

Commit subjects follow [Conventional Commits](https://www.conventionalcommits.org/):
`type(scope): summary`, with type one of `feat`, `fix`, `perf`, `refactor`, `docs`, `test`,
`build`, `ci`, `chore`, `style` or `revert`, and `!` after the type for a breaking change. CI
checks every new commit (`.github/workflows/commits.yml`); the changelog is generated from them.

Releases: `git cliff --unreleased --tag vX.Y.Z --prepend CHANGELOG.md` ([git-cliff](https://git-cliff.org),
config in `cliff.toml`), edit the new section by hand if needed, bump the version in `Cargo.toml`,
commit as `chore(release): vX.Y.Z`, then tag `vX.Y.Z` and push the tag. The release workflow
publishes the tag's `CHANGELOG.md` section as the release notes.

Branches and CI: work goes to `dev`; at a release `dev` is merged into `main` with `--no-ff` and
the merge is tagged. GitHub Actions run only on release tags (commit-subject check, build and
tests, release) and by hand (`workflow_dispatch`), never on branch pushes, so run the checks
locally before pushing: `cargo fmt --all --check`,
`cargo clippy --workspace --all-targets --features einst-plugin/standalone -- -D warnings` and
`cargo test --workspace --no-fail-fast`.

dreist-commons: the shared crates are the submodule at `commons` (clone with `--recursive`). To
change them, commit and push in `commons/` first, then commit the new pointer here as
`build: dreist-commons <short sha or vX.Y.Z>`; goldens must stay identical unless the change
means to alter them (say why in the commit).
