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

if ! grep -Fq -- '--workflow test-and-build.yml' "$GATE_SCRIPT" ||
   ! grep -Fq -- 'select(.headSha == \"${COMMIT_SHA}\" and (.event == \"push\" or .event == \"workflow_dispatch\"))' "$GATE_SCRIPT"; then
    printf '[ERROR] Version-bump CI gate must select same-source push or dispatch runs.\n' >&2
    exit 1
fi

if ! grep -Fq -- 'python3 scripts/release/verify-ci-run.py --sha "$COMMIT_SHA" --run-id "$RUN_ID" "${EXTRA[@]}"' "$GATE_SCRIPT"; then
    printf '[ERROR] Version-bump CI gate must verify the selected run and source.\n' >&2
    exit 1
fi

printf '[OK]    Version-bump CI gate allows the full three-platform release window.\n'
