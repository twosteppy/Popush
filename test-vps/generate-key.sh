#!/usr/bin/env bash
# Generate a throwaway ed25519 key pair for the test VPS (§23.3). This key is used
# ONLY by the integration harness against the container, it never touches the
# developer's ~/.ssh (Agent Rule 6).
set -euo pipefail
cd "$(dirname "$0")"

if [[ -f test_key ]]; then
  echo "test_key already exists; refusing to overwrite (delete it first)." >&2
  exit 1
fi

ssh-keygen -t ed25519 -f test_key -N "" -C "popush-integration-test"
cp test_key.pub authorized_keys
echo "Wrote ./test_key, ./test_key.pub, and ./authorized_keys."
echo "These are test-only credentials for the throwaway container. Do not reuse."
