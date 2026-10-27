param(
    [ValidateSet('original', 'original-before', 'dx12-visual', 'original-after')]
    [string]$CaseLabel = 'original'
)
$ErrorActionPreference = 'Stop'
$controller = Join-Path $env:GITHUB_WORKSPACE 'controller'
$metadata = Join-Path $env:RUNNER_TEMP "windows-startup-diagnostics-$CaseLabel.json"
$identity = Get-Content -LiteralPath $metadata -Raw | ConvertFrom-Json
$expectedCase = if ($CaseLabel -eq 'dx12-visual') { 'dx12-presentation' } else { 'original' }
if ($identity.source_commit -ne $env:WINDOWS_SOURCE_COMMIT -or
    [string]$identity.github_run_id -ne $env:WINDOWS_RUN_ID -or
    [string]$identity.github_run_attempt -ne $env:WINDOWS_RUN_ATTEMPT -or
    $identity.diagnostic_case -ne $expectedCase -or $identity.run_label -ne $CaseLabel) {
    throw 'Wait-stack trace input differs from validated source run'
}
$expectedPresentation = if ($CaseLabel -eq 'dx12-visual') { 'DxgiFromVisual' } else { $null }
if ($identity.dx12_presentation_system -ne $expectedPresentation -or
    $null -ne $identity.dx12_environment_value) {
    throw 'Wait-stack trace presentation settings differ from the selected case'
}
$executable = Join-Path $env:RUNNER_TEMP "katana-windows-smoke-$CaseLabel/KatanA.exe"
$actualHash = (Get-FileHash -LiteralPath $executable -Algorithm SHA256).Hash.ToLowerInvariant()
if ($actualHash -ne $identity.main_sha256) { throw 'Wait-stack executable differs from validated payload' }
$output = Join-Path $env:GITHUB_WORKSPACE "target/windows-startup-diagnostics/$CaseLabel/wait-stacks-trace"
New-Item -ItemType Directory -Path $output -Force | Out-Null
Copy-Item -LiteralPath $metadata -Destination (Join-Path $output 'validated-identity.json')
# WHY: Windows待機開始の追加採取を元200ms検査とは別の所有processに限定する。
$settingName = 'WGPU_DX12_PRESENTATION_SYSTEM'
$waitSettingName = 'WGPU_DX12_USE_FRAME_LATENCY_WAITABLE_OBJECT'
$hadOriginalSetting = Test-Path -LiteralPath "Env:$settingName"
$originalSetting = [Environment]::GetEnvironmentVariable($settingName, 'Process')
$hadOriginalWaitSetting = Test-Path -LiteralPath "Env:$waitSettingName"
$originalWaitSetting = [Environment]::GetEnvironmentVariable($waitSettingName, 'Process')
try {
    if ($env:WINDOWS_COMPARISON -eq 'dx12-presentation') {
        Remove-Item -LiteralPath "Env:$waitSettingName" -ErrorAction SilentlyContinue
        if ($null -eq $expectedPresentation) {
            Remove-Item -LiteralPath "Env:$settingName" -ErrorAction SilentlyContinue
        } else {
            [Environment]::SetEnvironmentVariable($settingName, $expectedPresentation, 'Process')
        }
    }
    & (Join-Path $controller 'scripts/release/trace-windows-startup.ps1') `
        -ExecutablePath $executable -DiagnosticsDirectory $output -WaitStacks
} finally {
    if ($env:WINDOWS_COMPARISON -eq 'dx12-presentation') {
        if ($hadOriginalWaitSetting) {
            [Environment]::SetEnvironmentVariable($waitSettingName, $originalWaitSetting, 'Process')
        } else {
            Remove-Item -LiteralPath "Env:$waitSettingName" -ErrorAction SilentlyContinue
        }
        if ($hadOriginalSetting) {
            [Environment]::SetEnvironmentVariable($settingName, $originalSetting, 'Process')
        } else {
            Remove-Item -LiteralPath "Env:$settingName" -ErrorAction SilentlyContinue
        }
    }
}
