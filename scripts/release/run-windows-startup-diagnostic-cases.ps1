$ErrorActionPreference = 'Stop'
Set-Location -LiteralPath (Join-Path $env:GITHUB_WORKSPACE 'controller')
. (Join-Path $env:GITHUB_WORKSPACE 'controller/scripts/release/windows-dx12-compiler-environment.ps1')
$payload = Join-Path $env:RUNNER_TEMP "windows-native-comparison-payload-$env:WINDOWS_RUN_ID-$env:WINDOWS_RUN_ATTEMPT"
$resultsRoot = Join-Path $env:GITHUB_WORKSPACE 'target/windows-startup-diagnostics'
New-Item -ItemType Directory -Path $resultsRoot -Force | Out-Null
Set-Location -LiteralPath $payload
$results = [System.Collections.Generic.List[object]]::new()
# WHY: 待機方式をケース間で漏らさず、既定値の比較を同じrunner上で保つ。
$settingName = 'WGPU_DX12_USE_FRAME_LATENCY_WAITABLE_OBJECT'
$presentationName = 'WGPU_DX12_PRESENTATION_SYSTEM'
$hadOriginalSetting = Test-Path -LiteralPath "Env:$settingName"
$originalSetting = [Environment]::GetEnvironmentVariable($settingName, 'Process')
$hadOriginalPresentation = Test-Path -LiteralPath "Env:$presentationName"
$originalPresentation = [Environment]::GetEnvironmentVariable($presentationName, 'Process')
$traceFailures = [System.Collections.Generic.List[object]]::new()
$compilerTraceFailures = [System.Collections.Generic.List[object]]::new()
$compilerAmbientError = $null
if ($env:WINDOWS_COMPARISON -eq 'dx12-compiler' -and
    (Test-Path -LiteralPath 'Env:WGPU_DX12_COMPILER')) {
    $compilerAmbientError = 'Ambient WGPU_DX12_COMPILER is set; refusing an ambiguous before/after comparison.'
    [ordered]@{
        source_commit = $env:WINDOWS_SOURCE_COMMIT
        source_run_id = $env:WINDOWS_RUN_ID
        source_run_attempt = $env:WINDOWS_RUN_ATTEMPT
        comparison_mode = $env:WINDOWS_COMPARISON
        diagnostic_only = $true
        original_required_smoke_outcome_unchanged = $true
        setup_error = $compilerAmbientError
        cases = @()
    } | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $resultsRoot 'summary.json') -Encoding utf8
    throw $compilerAmbientError
}
$metadataNames = if ($env:WINDOWS_COMPARISON -eq 'dx12-waiting') {
    @('original-before', 'dx12-dont-wait', 'dx12-none', 'original-after')
} elseif ($env:WINDOWS_COMPARISON -eq 'wait-stacks') { @('original') }
elseif ($env:WINDOWS_COMPARISON -eq 'dx12-presentation') { @('original-before', 'dx12-visual', 'original-after') }
elseif ($env:WINDOWS_COMPARISON -eq 'dx12-compiler') { @('original-before', 'dx12-dxc', 'original-after') }
else { @('original', 'reduced-native-trace') }

foreach ($caseLabel in $metadataNames) {
    $caseDirectory = Join-Path $resultsRoot $caseLabel
    New-Item -ItemType Directory -Path $caseDirectory -Force | Out-Null
    $caseScript = Join-Path $env:RUNNER_TEMP "windows-startup-diagnostics-$caseLabel.ps1"
    $caseMetadataPath = Join-Path $env:RUNNER_TEMP "windows-startup-diagnostics-$caseLabel.json"
    $caseMetadata = Get-Content -LiteralPath $caseMetadataPath -Raw | ConvertFrom-Json
    $caseName = $caseMetadata.diagnostic_case
    Copy-Item -LiteralPath $caseMetadataPath -Destination (Join-Path $caseDirectory 'validated-identity.json')
    $caseOutput = Join-Path $caseDirectory 'github-output.txt'
    $consoleLog = Join-Path $caseDirectory 'powershell-console.log'
    [System.IO.File]::WriteAllText($caseOutput, '')
    $env:GITHUB_OUTPUT = $caseOutput
    $env:KATANA_STARTUP_DIAGNOSTICS_CASE = $caseLabel
    $caseFailure = $null
    $compilerEnvironment = $null
    try {
        if ($null -eq $caseMetadata.dx12_environment_value) {
            Remove-Item -LiteralPath "Env:$settingName" -ErrorAction SilentlyContinue
        } else {
            [Environment]::SetEnvironmentVariable($settingName, [string]$caseMetadata.dx12_environment_value, 'Process')
        }
        if ($env:WINDOWS_COMPARISON -eq 'dx12-presentation') {
            if ($null -eq $caseMetadata.dx12_presentation_system) {
                Remove-Item -LiteralPath "Env:$presentationName" -ErrorAction SilentlyContinue
            } else {
                [Environment]::SetEnvironmentVariable($presentationName, [string]$caseMetadata.dx12_presentation_system, 'Process')
            }
        }
        if ($env:WINDOWS_COMPARISON -eq 'dx12-compiler' -and $caseLabel -eq 'dx12-dxc') {
            $compilerEnvironment = Enable-WindowsDx12Compiler -Metadata $caseMetadata `
                -ControllerRoot (Join-Path $env:GITHUB_WORKSPACE 'controller')
        }
        & { . $caseScript } *> $consoleLog
    }
    catch { $caseFailure = $_.Exception.Message }
    finally {
        if ($hadOriginalSetting) {
            [Environment]::SetEnvironmentVariable($settingName, $originalSetting, 'Process')
        } else {
            Remove-Item -LiteralPath "Env:$settingName" -ErrorAction SilentlyContinue
        }
        if ($env:WINDOWS_COMPARISON -eq 'dx12-presentation') {
            if ($hadOriginalPresentation) {
                [Environment]::SetEnvironmentVariable($presentationName, $originalPresentation, 'Process')
            } else {
                Remove-Item -LiteralPath "Env:$presentationName" -ErrorAction SilentlyContinue
            }
        }
        if ($env:WINDOWS_COMPARISON -eq 'dx12-compiler') {
            if ($null -ne $compilerEnvironment) {
                Restore-WindowsDx12CompilerEnvironment -State $compilerEnvironment
            }
        }
    }
    $diagnosticsPath = $null
    $outputLine = Get-Content -LiteralPath $caseOutput | Where-Object { $_ -like 'diagnostics_directory=*' } | Select-Object -Last 1
    if ($null -ne $outputLine) { $diagnosticsPath = $outputLine.Substring('diagnostics_directory='.Length) }
    if (-not $diagnosticsPath -or -not (Test-Path -LiteralPath $diagnosticsPath -PathType Container)) {
        if (-not $caseFailure) { $caseFailure = 'Smoke did not retain its raw diagnostics directory' }
    } else {
        $rawDirectory = Join-Path $caseDirectory 'raw'
        New-Item -ItemType Directory -Path $rawDirectory -Force | Out-Null
        Copy-Item -Path (Join-Path $diagnosticsPath '*') -Destination $rawDirectory -Recurse -Force
    }
    $exePath = Join-Path $env:RUNNER_TEMP "katana-windows-smoke-$caseLabel/KatanA.exe"
    $actualExeSha256 = $null
    if (Test-Path -LiteralPath $exePath -PathType Leaf) {
        $actualExeSha256 = (Get-FileHash -LiteralPath $exePath -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($actualExeSha256 -ne $caseMetadata.main_sha256) {
            $caseFailure = 'Extracted executable hash differs from validated payload identity'
        }
    } elseif (-not $caseFailure) { $caseFailure = 'Smoke extraction did not retain KatanA.exe' }
    $caseOutcome = if ($caseFailure) { 'failure' } else { 'success' }
    $result = [ordered]@{
        diagnostic_case = $caseName
        run_label = $caseLabel
        dx12_environment_value = $caseMetadata.dx12_environment_value
        dx12_presentation_system = $caseMetadata.dx12_presentation_system
        dx12_compiler = $caseMetadata.dx12_compiler
        dx12_compiler_receipt_sha256 = $caseMetadata.dx12_compiler_receipt_sha256
        source_commit = $caseMetadata.source_commit
        source_run_id = $caseMetadata.github_run_id
        source_run_attempt = $caseMetadata.github_run_attempt
        archive_sha256 = $caseMetadata.archive_sha256
        original_powershell_body_sha256 = $caseMetadata.powershell_body_sha256
        diagnostic_powershell_body_sha256 = $caseMetadata.diagnostic_body_sha256
        expected_main_sha256 = $caseMetadata.main_sha256
        actual_executable_sha256 = $actualExeSha256
        diagnostics_directory = $diagnosticsPath
        outcome = $caseOutcome
        error = $caseFailure
    }
    $results.Add($result)
    $resultPath = Join-Path $caseDirectory 'result.json'
    $result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $resultPath -Encoding utf8
    $caseTraceFailure = $null
    if ($env:WINDOWS_COMPARISON -eq 'dx12-presentation' -and $caseLabel -in @('original-before', 'dx12-visual')) {
        try {
            & (Join-Path $env:GITHUB_WORKSPACE 'controller/scripts/release/capture-windows-startup-wait-stacks.ps1') `
                -CaseLabel $caseLabel
        } catch {
            $caseTraceFailure = $_.Exception.Message
            $traceFailures.Add([ordered]@{ run_label = $caseLabel; error = $caseTraceFailure })
        }
        if ($caseTraceFailure) {
            $result.wait_stack_trace_error = $caseTraceFailure
            $result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $resultPath -Encoding utf8
        }
    }
    if ($env:WINDOWS_COMPARISON -eq 'dx12-compiler' -and
        $caseLabel -in @('original-before', 'dx12-dxc')) {
        try {
            & (Join-Path $env:GITHUB_WORKSPACE 'controller/scripts/release/capture-windows-startup-wait-stacks.ps1') `
                -CaseLabel $caseLabel
        } catch {
            $compilerTraceFailures.Add([ordered]@{ run_label = $caseLabel; error = $_.Exception.Message })
            $result.dx12_compiler_trace_error = $_.Exception.Message
            $result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $resultPath -Encoding utf8
        }
    }
}

$traceFailure = $null
if ($env:WINDOWS_COMPARISON -eq 'wait-stacks') {
    # WHY: 元smokeの結果を保持した後で別processを採取し、採取結果で本体を合格にしない。
    try {
        & (Join-Path $env:GITHUB_WORKSPACE 'controller/scripts/release/capture-windows-startup-wait-stacks.ps1')
    } catch { $traceFailure = $_.Exception.Message }
}

$summary = [ordered]@{
    source_commit = $env:WINDOWS_SOURCE_COMMIT
    source_run_id = $env:WINDOWS_RUN_ID
    source_run_attempt = $env:WINDOWS_RUN_ATTEMPT
    comparison_mode = $env:WINDOWS_COMPARISON
    diagnostic_only = $true
    original_required_smoke_outcome_unchanged = $true
    wait_stack_trace_error = $traceFailure
    presentation_wait_stack_trace_errors = @($traceFailures)
    dx12_compiler_trace_errors = @($compilerTraceFailures)
    cases = @($results)
}
$summary | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $resultsRoot 'summary.json') -Encoding utf8
if ($traceFailure -or $compilerTraceFailures.Count -gt 0 -or $traceFailures.Count -gt 0 -or
    @($results | Where-Object { $_.outcome -ne 'success' }).Count -gt 0) {
    throw 'One or more diagnostic smoke cases failed; per-case evidence was retained'
}
Write-Host 'All diagnostic smoke cases were attempted and their payload-bound results retained.'
