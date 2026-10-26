#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
WORKFLOW="${ROOT_DIR}/.github/workflows/build-and-release.yml"
SMOKE_SCRIPT="${ROOT_DIR}/scripts/release/smoke-launch-packaged-app.sh"
HEARTBEAT_HELPER="${ROOT_DIR}/scripts/release/startup-heartbeat.sh"
IDENTITY_HELPER="${ROOT_DIR}/scripts/release/packaged-process-identity.sh"

assert_contains() {
    local path="$1"
    local expected="$2"
    if ! grep -Fq -- "$expected" "$path"; then
        echo "FAIL: missing packaged startup contract in ${path}: ${expected}" >&2
        exit 1
    fi
}

assert_not_contains() {
    local path="$1"
    local unexpected="$2"
    if grep -Fq -- "$unexpected" "$path"; then
        echo "FAIL: paid Apple distribution requirement found in ${path}: ${unexpected}" >&2
        exit 1
    fi
}

assert_build_linux_contains() {
    local expected="$1"
    if ! awk '
        /^  build_linux:$/ { in_build_linux=1; next }
        in_build_linux && /^  [[:alnum:]_]+:$/ { exit }
        in_build_linux { print }
    ' "$WORKFLOW" | grep -Fq -- "$expected"; then
        echo "FAIL: missing build_linux startup contract in ${WORKFLOW}: ${expected}" >&2
        exit 1
    fi
}

assert_build_linux_step_contains() {
    local step="$1"
    local expected="$2"
    if ! awk -v step="$step" '
        /^  build_linux:$/ { in_build_linux=1; next }
        in_build_linux && /^  [[:alnum:]_]+:$/ { exit }
        in_build_linux && $0 == "      - name: " step { in_step=1; next }
        in_build_linux && in_step && /^      - name:/ { exit }
        in_build_linux && in_step { print }
    ' "$WORKFLOW" | grep -Fq -- "$expected"; then
        echo "FAIL: missing build_linux step contract in ${WORKFLOW}: step=${step}; expected=${expected}" >&2
        exit 1
    fi
}

assert_build_windows_step_contains() {
    local step="$1"
    local expected="$2"
    if ! awk -v step="$step" '
        /^  build_windows:$/ { in_build_windows=1; next }
        in_build_windows && /^  [[:alnum:]_]+:$/ { exit }
        in_build_windows && $0 == "      - name: " step { in_step=1; next }
        in_build_windows && in_step && /^      - name:/ { exit }
        in_build_windows && in_step { print }
    ' "$WORKFLOW" | grep -Fq -- "$expected"; then
        echo "FAIL: missing build_windows step contract in ${WORKFLOW}: step=${step}; expected=${expected}" >&2
        exit 1
    fi
}

bash -n "$SMOKE_SCRIPT"
bash -n "$HEARTBEAT_HELPER"
bash -n "$IDENTITY_HELPER"
PYTHONDONTWRITEBYTECODE=1 python3 "${ROOT_DIR}/scripts/release/test-macos-stale-dmg.py"
bash "${ROOT_DIR}/scripts/release/test-startup-evidence-retention.sh"
bash "${ROOT_DIR}/scripts/release/test-packaged-process-identity.sh"
bash -n "${ROOT_DIR}/scripts/build/package-mac.sh"
HEARTBEAT=$(mktemp -t katana-heartbeat-contract.XXXXXX)
cleanup_heartbeat_contract() { rm -f "$HEARTBEAT"; }
trap cleanup_heartbeat_contract EXIT
source "$HEARTBEAT_HELPER"

printf 'first-frame\nframe=1\n' >"$HEARTBEAT"
if require_heartbeat_advance "$HEARTBEAT" 1; then
    echo "FAIL: stalled heartbeat regression accepted a static frame" >&2
    exit 1
fi
printf 'first-frame\n' >"$HEARTBEAT"
if require_heartbeat_advance "$HEARTBEAT" 1; then
    echo "FAIL: heartbeat regression accepted a partial update" >&2
    exit 1
fi
printf 'first-frame\nframe=2\n' >"$HEARTBEAT"
advanced_frame=$(require_heartbeat_advance "$HEARTBEAT" 1)
if require_heartbeat_advance "$HEARTBEAT" "$advanced_frame"; then
    echo "FAIL: heartbeat regression accepted a mid-observation stall" >&2
    exit 1
fi
PYTHONDONTWRITEBYTECODE=1 python3 \
    "${ROOT_DIR}/scripts/release/test-verify-binary-architecture.py"

assert_contains "$WORKFLOW" "runner: macos-15"
assert_contains "$WORKFLOW" "runner: macos-15-intel"
assert_contains "$WORKFLOW" "mode: macos-arm64"
assert_contains "$WORKFLOW" "mode: macos-x86_64"
assert_contains "$WORKFLOW" "needs: [preflight, build_macos, smoke_macos, build_linux, build_windows]"
assert_contains "$WORKFLOW" "needs.build_macos.result == 'success' && needs.smoke_macos.result == 'success' && needs.build_linux.result == 'success' && needs.build_windows.result == 'success'"
assert_contains "$WORKFLOW" '"$GITHUB_WORKSPACE/scripts/release/smoke-launch-packaged-app.sh" linux-x86_64'
assert_build_linux_contains 'libxkbcommon-x11-0'
assert_build_linux_step_contains 'Install dependencies' 'mesa-vulkan-drivers'
assert_build_linux_step_contains 'Install dependencies' 'vulkan-tools'
assert_build_linux_step_contains 'Verify software Vulkan adapter' 'lvp_icd=$(find /usr/share/vulkan/icd.d'
assert_build_linux_step_contains 'Verify software Vulkan adapter' "name '*lvp*.json'"
assert_build_linux_step_contains 'Verify software Vulkan adapter' 'LIBGL_ALWAYS_SOFTWARE=1 VK_DRIVER_FILES="$lvp_icd" WGPU_BACKEND=vulkan'
assert_build_linux_step_contains 'Verify software Vulkan adapter' 'xvfb-run -a vulkaninfo --summary'
assert_build_linux_step_contains 'Smoke launch packaged Linux app' 'lvp_icd=$(find /usr/share/vulkan/icd.d'
assert_build_linux_step_contains 'Smoke launch packaged Linux app' 'export LIBGL_ALWAYS_SOFTWARE=1 VK_DRIVER_FILES="$lvp_icd" WGPU_BACKEND=vulkan'
assert_build_linux_step_contains 'Smoke launch packaged Linux app' 'id: linux_startup_smoke'
assert_build_linux_step_contains 'Upload Linux startup diagnostics' "if: failure() && steps.linux_startup_smoke.outcome == 'failure'"
assert_build_linux_step_contains 'Upload Linux startup diagnostics' 'tmp/trash/*-startup.*/heartbeat.txt'
assert_build_linux_step_contains 'Upload Linux startup diagnostics' 'tmp/trash/*-startup.*/startup.log'
assert_contains "$WORKFLOW" 'cd "$APP_DIR"'
assert_contains "$WORKFLOW" 'cd "$TEMP_DIR"'
assert_contains "$WORKFLOW" 'Start-Process -FilePath $expectedExecutable -WorkingDirectory $directory'
assert_build_windows_step_contains 'Smoke launch packaged Windows app' '$stdoutLog = New-TemporaryFile'
assert_build_windows_step_contains 'Smoke launch packaged Windows app' '-RedirectStandardOutput $stdoutLog.FullName -RedirectStandardError $log.FullName'
assert_build_windows_step_contains 'Smoke launch packaged Windows app' '$env:RUST_LOG = "warn,katana_ui=info,katana_core=info,KatanA=info,eframe::native::run=trace,eframe::native::wgpu_integration=trace,egui_wgpu=debug,winit::platform_impl::windows::event_loop=trace"'
assert_build_windows_step_contains 'Smoke launch packaged Windows app' '$startupFailure = $_'
assert_build_windows_step_contains 'Smoke launch packaged Windows app' 'Get-Item -LiteralPath $heartbeat.FullName'
assert_build_windows_step_contains 'Smoke launch packaged Windows app' 'Select-Object Id, Path, StartTime, CPU, WorkingSet64, MainWindowHandle, MainWindowTitle, Responding'
assert_build_windows_step_contains 'Smoke launch packaged Windows app' 'throw $startupFailure'
assert_build_windows_step_contains 'Smoke launch packaged Windows app' 'Start-Sleep -Milliseconds 200'
assert_build_windows_step_contains 'Smoke launch packaged Windows app' 'foreach ($attempt in 1..10)'
assert_build_windows_step_contains 'Smoke launch packaged Windows app' 'if ($peakRss -gt 512MB)'
assert_build_windows_step_contains 'Smoke launch packaged Windows app' 'if ($fontBytes -gt 134217728)'
assert_build_windows_step_contains 'Upload Windows startup diagnostics' "if: failure() && steps.windows_startup_smoke.outcome == 'failure'"
assert_build_windows_step_contains 'Upload Windows startup diagnostics' 'path: ${{ steps.windows_startup_smoke.outputs.diagnostics_directory }}'
assert_contains "$WORKFLOW" "Office worker is live in an empty workspace"
assert_contains "$WORKFLOW" "UI heartbeat stalled after first frame"
assert_contains "$WORKFLOW" 'function Assert-PackagedProcessIdentity'
assert_contains "$WORKFLOW" 'Get-Process -Id $process.Id -ErrorAction Stop'
assert_contains "$WORKFLOW" 'Get-FileHash -LiteralPath $actualExecutable -Algorithm SHA256'
assert_contains "$WORKFLOW" 'Packaged process image path mismatch'
assert_contains "$WORKFLOW" 'Packaged process image hash changed during smoke test'
assert_contains "$WORKFLOW" 'packaged_identity_verified=true; executable=$expectedExecutable'
assert_contains "$WORKFLOW" "- name: Codesign .app"
assert_contains "$WORKFLOW" 'codesign --force --deep --sign - "$APP_PATH"'
assert_contains "$WORKFLOW" 'ditto -c -k --keepParent "$APP_PATH" "$ZIP_PATH"'
assert_contains "$WORKFLOW" 'mv "$ZIP_PATH" "$STALE_DIR/"'
assert_contains "$SMOKE_SCRIPT" '[[ "$(uname -m)" == "arm64" ]]'
assert_contains "$SMOKE_SCRIPT" '[[ "$(uname -m)" == "x86_64" ]]'
assert_contains "$SMOKE_SCRIPT" 'startup-heartbeat.sh'
assert_contains "$SMOKE_SCRIPT" 'packaged-process-identity.sh'
assert_contains "$SMOKE_SCRIPT" 'packaged_find_matching_pid'
assert_contains "$SMOKE_SCRIPT" 'packaged_verify_process_identity'
assert_contains "$IDENTITY_HELPER" 'packaged_process_image'
assert_contains "$HEARTBEAT_HELPER" 'require_heartbeat_advance'
assert_contains "$HEARTBEAT_HELPER" 'UI heartbeat stalled after first frame'
assert_not_contains "$WORKFLOW" 'APPLE_CERTIFICATE_P12_BASE64'
assert_not_contains "$WORKFLOW" 'APPLE_APP_SPECIFIC_PASSWORD'
assert_not_contains "$WORKFLOW" 'notarize'

echo "PASS: packaged startup contract covers native macOS arm64/x86_64, Linux, and Windows"
