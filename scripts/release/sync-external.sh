#!/bin/bash
set -euo pipefail

# Usage: ./scripts/release/sync-external.sh <version> <tag> <artifacts_dir>
VERSION=${1:-}
TAG=${2:-}
ARTIFACTS_DIR=${3:-}

if [[ -z "$VERSION" || -z "$TAG" || -z "$ARTIFACTS_DIR" ]]; then
    echo "Usage: $0 <version> <tag> <artifacts_dir>" >&2
    exit 1
fi

RESULT_FILE="${ARTIFACTS_DIR}/external-registry-sync.json"
TARGET_RESULTS=()
FAILURE_COUNT=0
WINGET_FAILURE=0

record_target() {
    local name=$1
    local status=$2
    local reason=$3
    TARGET_RESULTS+=("${name}"$'\t'"${status}"$'\t'"${reason}"$'\t'"${4:-}"$'\t'"${5:-}")
    if [[ "$status" == "failed" ]]; then
        FAILURE_COUNT=$((FAILURE_COUNT + 1))
        if [[ "$name" == "winget" ]]; then
            WINGET_FAILURE=1
        fi
        echo "⚠️ ${name}: ${reason}"
    elif [[ "$status" == "skipped" ]]; then
        echo "⏭️ ${name}: ${reason}"
    elif [[ "$name" == "winget" ]]; then
        echo "✅ WinGet: submission completed"
    else
        echo "✅ ${name}: synchronized"
    fi
}

run_brew_updater() {
    local target=$1 artifact=$2 sha256=$3 updater=$4
    local updater_output updater_status http_status
    if updater_output=$("$updater" "$TAG" "$sha256" "$(basename "$artifact")" 2>&1); then
        record_target "$target" success updated
    else
        updater_status=$?
        http_status=""
        # WHY: helperの生出力には機密が混入し得るため、数値の失敗情報だけを残す。
        if [[ "$updater_output" =~ \(HTTP\ ([1-5][0-9][0-9])\) ]]; then
            http_status="${BASH_REMATCH[1]}"
        fi
        record_target "$target" failed updater_failed "$updater_status" "$http_status"
        echo "${target} updater diagnostic: exit=${updater_status}, http=${http_status:-unknown}"
    fi
    unset updater_output
}

sync_brew_target() {
    local target=$1
    local artifact_pattern=$2
    local updater=$3
    local token=${HOMEBREW_KATANA_GIT_TOKEN:-}
    local artifact sha256

    if [[ -z "$token" ]]; then
        record_target "$target" skipped token_not_configured
        return
    fi
    if [[ ! -f "$updater" ]]; then
        record_target "$target" failed updater_unavailable
        return
    fi

    artifact=$(find "$ARTIFACTS_DIR" -maxdepth 1 -type f -name "$artifact_pattern" -print | sort -V | tail -1)
    if [[ -z "$artifact" ]]; then
        record_target "$target" skipped artifact_missing
        return
    fi
    if ! sha256=$(shasum -a 256 "$artifact" 2>/dev/null | awk '{print $1}') || [[ -z "$sha256" ]]; then
        record_target "$target" failed checksum_failed
        return
    fi
    run_brew_updater "$target" "$artifact" "$sha256" "$updater"
}

mkdir -p "$ARTIFACTS_DIR"
echo "🌏 Syncing with external registries for ${TAG}..."

sync_brew_target homebrew 'KatanA-Desktop-*.dmg' ./scripts/release/update-homebrew.sh
sync_brew_target linuxbrew 'KatanA-linux-*.tar.gz' ./scripts/release/update-linuxbrew.sh

PACKAGE_ID="HiroyukiFuruno.katana-desktop"
MSI_URL="https://github.com/HiroyukiFuruno/KatanA/releases/download/${TAG}/KatanA-windows-x86_64.msi"

if [[ -z "${WINGET_GH_TOKEN:-}" ]]; then
    record_target winget skipped token_not_configured
elif [[ ! -f "${ARTIFACTS_DIR}/KatanA-windows-x86_64.msi" ]]; then
    record_target winget skipped msi_artifact_missing
else
    if ! command -v komac >/dev/null 2>&1; then
        brew install komac >/dev/null 2>&1 || true
    fi

    if ! command -v komac >/dev/null 2>&1; then
        record_target winget failed komac_unavailable
    elif komac list "$PACKAGE_ID" --token "$WINGET_GH_TOKEN" >/dev/null 2>&1; then
        if update_output=$(komac update "$PACKAGE_ID" \
            --version "$VERSION" \
            --urls "$MSI_URL" \
            --release-notes-url "https://github.com/HiroyukiFuruno/KatanA/releases/tag/${TAG}" \
            --submit \
            --token "$WINGET_GH_TOKEN" 2>&1); then
            record_target winget success submission_completed
        elif [[ "$update_output" == *'does not have the correct permissions to execute `CreateRef`'* ]]; then
            record_target winget failed create_ref_permission_denied
        else
            record_target winget failed update_failed
        fi
        unset update_output
    else
        record_target winget failed package_lookup_failed
    fi
fi

python3 - "$RESULT_FILE" "$VERSION" "$TAG" "${TARGET_RESULTS[@]}" <<'PY'
import json
import pathlib
import sys

result_path = pathlib.Path(sys.argv[1])
version, tag = sys.argv[2:4]
targets = []
for value in sys.argv[4:]:
    name, status, reason, helper_exit, http_status = value.split("\t", 4)
    target = {"name": name, "status": status, "reason": reason}
    if helper_exit:
        target["updater_exit_code"] = int(helper_exit)
        target["http_status"] = int(http_status) if http_status else None
    if reason == "create_ref_permission_denied":
        target["guidance"] = (
            "Check whether the WinGet fork is synchronized and the credential can create refs. "
            "See https://github.com/russellbanks/Komac/issues/1142."
        )
    targets.append(target)

result_path.write_text(
    json.dumps(
        {"version": version, "tag": tag, "targets": targets},
        ensure_ascii=False,
        indent=2,
    )
    + "\n",
    encoding="utf-8",
)
PY

if [[ -n "${GITHUB_STEP_SUMMARY:-}" ]]; then
    python3 - "$GITHUB_STEP_SUMMARY" "${TARGET_RESULTS[@]}" <<'PY'
import pathlib
import sys

summary_path = pathlib.Path(sys.argv[1])
rows = [value.split("\t", 4) for value in sys.argv[2:]]
with summary_path.open("a", encoding="utf-8") as summary:
    summary.write("\n### External registry sync\n\n")
    summary.write("| Target | Result | Detail |\n| --- | --- | --- |\n")
    for name, status, reason, *_diagnostics in rows:
        detail = reason
        if reason == "create_ref_permission_denied":
            detail = (
                "Check WinGet fork synchronization and credential access; "
                "[Komac issue #1142](https://github.com/russellbanks/Komac/issues/1142)"
            )
        summary.write(f"| {name} | {status} | {detail} |\n")
PY
fi

if (( FAILURE_COUNT > 0 )); then
    if (( WINGET_FAILURE > 0 )); then
        echo "::warning title=WinGet同期失敗::The optional WinGet update failed; see the per-target results and artifact."
    else
        echo "::warning title=External registry sync failed::One or more optional registry updates failed; see the per-target results and artifact."
    fi
    echo "⚠️ External registry sync completed with ${FAILURE_COUNT} failed target(s); product release is not blocked."
else
    echo "External registry sync completed; see per-target results for success and skipped targets."
fi
echo "External registry result: ${RESULT_FILE}"
