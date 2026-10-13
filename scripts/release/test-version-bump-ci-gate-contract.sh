#!/bin/bash
set -euo pipefail

GATE_SCRIPT=scripts/release/verify-version-bump-ci.sh

if ! grep -Fq 'MAX_ATTEMPTS=${CI_GATE_WAIT_ATTEMPTS:-360}' "$GATE_SCRIPT"; then
    printf '[ERROR] Version-bump CI gate must default to 360 attempts.\n' >&2
    exit 1
fi

if ! grep -Fq 'WAIT_SECONDS=${CI_GATE_WAIT_SECONDS:-30}' "$GATE_SCRIPT"; then
    printf '[ERROR] Version-bump CI gate must retain its 30-second polling interval.\n' >&2
    exit 1
fi

if ! grep -Fq -- '--event push' "$GATE_SCRIPT"; then
    printf '[ERROR] Version-bump CI gate must verify the post-merge push workflow.\n' >&2
    exit 1
fi

printf '[OK]    Version-bump CI gate allows the full three-platform release window.\n'
