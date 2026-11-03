# Git conventions

## Commits

- Subjects follow [Conventional Commits](https://www.conventionalcommits.org/):
  `type(scope): summary`, imperative, lower-case summary, no trailing period.
  Types: `feat`, `fix`, `perf`, `refactor`, `docs`, `test`, `build`, `ci`, `chore`,
  `style`, `revert`; `!` after the type (or a `BREAKING CHANGE:` footer) for a breaking change.
- Scopes are optional and short: a crate or area (`einst`, `einst-formats`, `einst-dsp`, `einst-plugin`, `ci`).
- A body only when the why is not obvious; wrap at 72 columns.
- CI checks every new commit subject (`.github/workflows/commits.yml`).
- No AI attribution of any kind: no `Co-Authored-By` for an assistant, no session trailers, no
  "Generated with ..." lines, in commits, PRs, tags or release notes. Author and committer are
  the repository's configured identity.

## Changelog and releases

- `CHANGELOG.md` is generated with [git-cliff](https://git-cliff.org) (`cliff.toml`) from the
  commit subjects: `git cliff --unreleased --tag vX.Y.Z --prepend CHANGELOG.md`, then edit the
  new section by hand where it helps.
- Release: bump `version` in `Cargo.toml`, commit `chore(release): vX.Y.Z`, tag `vX.Y.Z`
  (annotated) and push the tag. The release workflow publishes the tag's `CHANGELOG.md`
  section as the release notes and attaches the macOS zips.

## dreist-commons submodule

The shared crates are the submodule at `commons` (Cargo uses them by path). Clone with
`--recursive`, or run `git submodule update --init`. To change them:

1. Commit and push in `commons/` (its own Conventional Commits), on its `main`.
2. Here: `git add commons` and commit `build: dreist-commons <short sha or vX.Y.Z>`.
3. Run this repository's checks; goldens must stay identical unless the change means to alter
   them (say why in the commit).
