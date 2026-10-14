#!/usr/bin/env bash
set -euo pipefail

project_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
debug_binary="${project_dir}/target/debug/servoshell"
release_binary="${project_dir}/target/release/servoshell"

if [[ -x "${debug_binary}" ]]; then
  exec "${debug_binary}" "$@"
fi

if [[ -x "${release_binary}" ]]; then
  exec "${release_binary}" "$@"
fi

printf 'LibreNav is not built yet. Run: ./mach build --dev --media-stack dummy\n' >&2
exit 1
