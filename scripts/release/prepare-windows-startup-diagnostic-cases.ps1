$ErrorActionPreference = 'Stop'
Set-Location -LiteralPath (Join-Path $env:GITHUB_WORKSPACE 'controller')
$payload = Join-Path $env:RUNNER_TEMP "windows-native-comparison-payload-$env:WINDOWS_RUN_ID-$env:WINDOWS_RUN_ATTEMPT"
$originalScript = Join-Path $env:RUNNER_TEMP 'windows-startup-validated-original.ps1'
$originalMetadata = Join-Path $env:RUNNER_TEMP 'windows-startup-validated-original.json'
$inputSource = Join-Path $env:GITHUB_WORKSPACE 'input-source'
python scripts/release/prepare-windows-startup-comparison.py `
    --payload-dir $payload `
    --workflow (Join-Path $inputSource '.github/workflows/build-and-release.yml') `
    --current-root $inputSource `
    --output-script $originalScript `
    --output-metadata $originalMetadata `
    --expected-version v0.22.42 `
    --expected-source-commit $env:WINDOWS_SOURCE_COMMIT `
    --expected-run-id $env:WINDOWS_RUN_ID `
    --expected-run-attempt $env:WINDOWS_RUN_ATTEMPT
if ($LASTEXITCODE -ne 0) { throw 'Validated original smoke preparation failed' }

if ($env:WINDOWS_COMPARISON -eq 'dx12-waiting') {
    $cases = @(
        @{ Case = 'original'; Label = 'original-before' },
        @{ Case = 'dx12-waiting'; Label = 'dx12-dont-wait'; Value = 'DontWait' },
        @{ Case = 'dx12-waiting'; Label = 'dx12-none'; Value = 'None' },
        @{ Case = 'original'; Label = 'original-after' }
    )
} elseif ($env:WINDOWS_COMPARISON -eq 'wait-stacks') {
    $cases = @(@{ Case = 'original'; Label = 'original' })
} elseif ($env:WINDOWS_COMPARISON -eq 'dx12-presentation') {
    $cases = @(
        @{ Case = 'original'; Label = 'original-before' },
        @{ Case = 'dx12-presentation'; Label = 'dx12-visual'; PresentationSystem = 'DxgiFromVisual' },
        @{ Case = 'original'; Label = 'original-after' }
    )
} else {
    $cases = @(
        @{ Case = 'original'; Label = 'original' },
        @{ Case = 'reduced-native-trace'; Label = 'reduced-native-trace' }
    )
}

foreach ($case in $cases) {
    $caseLabel = [string]$case['Label']
    $caseType = [string]$case['Case']
    $casePresentation = $case['PresentationSystem']
    $caseScript = Join-Path $env:RUNNER_TEMP "windows-startup-diagnostics-$caseLabel.ps1"
    $caseMetadata = Join-Path $env:RUNNER_TEMP "windows-startup-diagnostics-$caseLabel.json"
    $extraArgs = @()
    if ($case.ContainsKey('Value')) { $extraArgs = @('--dx12-value', $case.Value) }
    if ($casePresentation) { $extraArgs += @('--presentation-system', $casePresentation) }
    python scripts/release/prepare-windows-startup-diagnostics.py `
        --source-script $originalScript `
        --source-metadata $originalMetadata `
        --output-script $caseScript `
        --output-metadata $caseMetadata `
        --expected-source-commit $env:WINDOWS_SOURCE_COMMIT `
        --expected-run-id $env:WINDOWS_RUN_ID `
        --expected-run-attempt $env:WINDOWS_RUN_ATTEMPT `
        --case $caseType `
        --run-label $caseLabel `
        @extraArgs
    if ($LASTEXITCODE -ne 0) { throw "Could not prepare diagnostic case: $caseLabel" }
}
