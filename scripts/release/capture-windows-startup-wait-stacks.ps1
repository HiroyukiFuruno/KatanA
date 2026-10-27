$ErrorActionPreference = 'Stop'
$controller = Join-Path $env:GITHUB_WORKSPACE 'controller'
$metadata = Join-Path $env:RUNNER_TEMP 'windows-startup-diagnostics-original.json'
$identity = Get-Content -LiteralPath $metadata -Raw | ConvertFrom-Json
if ($identity.source_commit -ne $env:WINDOWS_SOURCE_COMMIT -or
    [string]$identity.github_run_id -ne $env:WINDOWS_RUN_ID -or
    [string]$identity.github_run_attempt -ne $env:WINDOWS_RUN_ATTEMPT) {
    throw 'Wait-stack trace input differs from validated source run'
}
$executable = Join-Path $env:RUNNER_TEMP 'katana-windows-smoke-original/KatanA.exe'
$actualHash = (Get-FileHash -LiteralPath $executable -Algorithm SHA256).Hash.ToLowerInvariant()
if ($actualHash -ne $identity.main_sha256) { throw 'Wait-stack executable differs from validated payload' }
$output = Join-Path $env:GITHUB_WORKSPACE 'target/windows-startup-diagnostics/wait-stacks-trace'
New-Item -ItemType Directory -Path $output -Force | Out-Null
Copy-Item -LiteralPath $metadata -Destination (Join-Path $output 'validated-identity.json')
# WHY: Windows待機開始の追加採取を元200ms検査とは別の所有processに限定する。
& (Join-Path $controller 'scripts/release/trace-windows-startup.ps1') `
    -ExecutablePath $executable -DiagnosticsDirectory $output -WaitStacks
