#!/usr/bin/env bash
# The commons submodule must point at a dreist-commons release: the commit
# one of its `v*` tags names (the merge into its main). Bumping it stays a
# pull request into dev (`build: dreist-commons vX.Y.Z`, or Dependabot's).
set -euo pipefail
git -C commons fetch -q --tags origin
if tag=$(git -C commons describe --exact-match --tags --match 'v[0-9]*' HEAD 2>/dev/null); then
  echo "commons is dreist-commons $tag"
else
  echo "::error::commons points at $(git -C commons rev-parse --short HEAD), not at a dreist-commons release: release dreist-commons first, then point commons at its tag (build: dreist-commons vX.Y.Z) through a pull request into dev."
  exit 1
fi
