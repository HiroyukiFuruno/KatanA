#!/usr/bin/env bash
# KatanA screenshot runner entry point.
#
# Builds the katana-screenshot binary (if needed) and executes a request file.
#
# Usage:
#   ./run.sh --request <request.json> --output <output_dir>
#
# Runs KatanaApp in-process with egui_kittest, not a packaged executable.
# Packaged startup is checked separately by scripts/release/smoke-launch-packaged-app.sh.
# Requires a Rust toolchain and a supported graphics backend.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
source "${SCRIPT_DIR}/build-environment.sh"
# WHY: 呼出元指定のCargo target rootへ全生成物を隔離し、既定配置は維持する。
BUILD_TARGET_DIR="${CARGO_TARGET_DIR:-${REPO_ROOT}/target}"
SCREENSHOT_TARGET_DIR="${BUILD_TARGET_DIR}/screenshot-harness"
EXTERNAL_HYPERLINK_FIXTURE="${BUILD_TARGET_DIR}/screenshot-fixtures/representative-with-external-hyperlink.pptx"
FIXTURE_TARGET_DIR="${BUILD_TARGET_DIR}/screenshot-fixtures"

if ! command -v cargo &>/dev/null; then
  echo "ERROR: cargo not found — install Rust via https://rustup.rs" >&2
  exit 1
fi

export SCRIPT_DIR REPO_ROOT BUILD_TARGET_DIR SCREENSHOT_TARGET_DIR EXTERNAL_HYPERLINK_FIXTURE FIXTURE_TARGET_DIR
# WHY: fixture作成前からrunner終了まで同じtarget leaseを保持し、未所有targetを拒否する。
exec python3 -B "${REPO_ROOT}/scripts/ci/cargo_target_init.py" --allow-external-target "${BUILD_TARGET_DIR}" --run bash -euo pipefail -c 'source /dev/stdin' screenshot-producer "$@" <<'PRODUCER'
# WHY: fixtureは親targetのleaseで保護し、既存fixtureを所有cacheと偽らずリンク先への書込みを拒否する。
python3 -c 'from pathlib import Path; import stat, sys
for index, item in enumerate(map(Path, sys.argv[1:])):
    if item.is_symlink() or (hasattr(item, "is_junction") and item.is_junction()):
        raise SystemExit(f"fixture path contains symlink or junction: {item}")
    try:
        metadata = item.lstat()
    except FileNotFoundError:
        continue
    if (index == 0 and not stat.S_ISDIR(metadata.st_mode)) or (index == 1 and (not stat.S_ISREG(metadata.st_mode) or metadata.st_nlink != 1)):
        raise SystemExit(f"fixture path is not an unshared regular destination: {item}")' "${FIXTURE_TARGET_DIR}" "${EXTERNAL_HYPERLINK_FIXTURE}"
python3 "${SCRIPT_DIR}/generate_external_hyperlink_pptx.py" \
  --source "${SCRIPT_DIR}/fixtures/v0-22-38-multi-format/representative.pptx" \
  --output "${EXTERNAL_HYPERLINK_FIXTURE}"

echo "[katana-screenshot] building runner..."
CARGO_TARGET_DIR="${BUILD_TARGET_DIR}" cargo build --locked --release --manifest-path "${REPO_ROOT}/Cargo.toml" --package katana-ui --bin kdv-office-worker --quiet
OFFICE_WORKER="${BUILD_TARGET_DIR}/release/kdv-office-worker"
if [[ -f "${OFFICE_WORKER}.exe" ]]; then
  OFFICE_WORKER="${OFFICE_WORKER}.exe"
fi
if [[ ! -x "${OFFICE_WORKER}" && ! -f "${OFFICE_WORKER}" ]]; then
  echo "ERROR: Office worker was not built at ${OFFICE_WORKER}" >&2
  exit 1
fi
export KATANA_KDV_OFFICE_WORKER="${OFFICE_WORKER}"
# WHY: 子targetだけのcleanもrunner終了まで拒否できるよう、親→子の順でleaseを保持する。
exec python3 -B "${REPO_ROOT}/scripts/ci/cargo_target_init.py" --allow-external-target "${SCREENSHOT_TARGET_DIR}" --run bash -euo pipefail -c 'source /dev/stdin' screenshot-child "$@" <<'CHILD_PRODUCER'
# 別lockfileの生成物を本体のtargetへ混在させず、実ファイル受入と同じ配置にする。
CARGO_TARGET_DIR="${SCREENSHOT_TARGET_DIR}" cargo build --locked --release --manifest-path "${SCRIPT_DIR}/Cargo.toml" --quiet

RUNNER="${SCREENSHOT_TARGET_DIR}/release/katana-screenshot"
if [[ -f "${RUNNER}.exe" ]]; then
  RUNNER="${RUNNER}.exe"
fi

exec "$RUNNER" "$@"
CHILD_PRODUCER
PRODUCER
