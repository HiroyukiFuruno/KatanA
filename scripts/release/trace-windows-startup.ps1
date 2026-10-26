[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$ExecutablePath,
    [Parameter(Mandatory = $true)]
    [string]$DiagnosticsDirectory
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$failure = $null
$wprStartAttempted = $false
$wprInstance = $null
$wprCommand = $null
$tracerptCommand = $null
$process = $null
$processStartTicks = $null
$resolvedExecutable = $null
$expectedSha256 = $null

function ConvertTo-NativeArgument {
    param([Parameter(Mandatory = $true)][AllowEmptyString()][string]$Value)
    $escaped = $Value -replace '(\\*)"', '$1$1\"'
    $escaped = $escaped -replace '(\\+)$', '$1$1'
    return '"' + $escaped + '"'
}

function Read-BoundedText {
    param([Parameter(Mandatory = $true)][string]$Path)
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) { return '' }
    $stream = [System.IO.File]::OpenRead($Path)
    try {
        $buffer = [byte[]]::new(1048576)
        $count = $stream.Read($buffer, 0, $buffer.Length)
        return [System.Text.Encoding]::UTF8.GetString($buffer, 0, $count)
    }
    finally { $stream.Dispose() }
}

function Invoke-BoundedNative {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][string[]]$Arguments,
        [Parameter(Mandatory = $true)][string]$Operation,
        [Parameter(Mandatory = $true)][string]$OutputDirectory,
        [Parameter(Mandatory = $true)][int]$TimeoutSeconds
    )
    $stdoutPath = Join-Path $OutputDirectory "$Operation.stdout.txt"
    $stderrPath = Join-Path $OutputDirectory "$Operation.stderr.txt"
    $exitPath = Join-Path $OutputDirectory "$Operation.exit-code.txt"
    $processPath = Join-Path $OutputDirectory "$Operation.process.json"
    $argumentLine = (($Arguments | ForEach-Object { ConvertTo-NativeArgument -Value $_ }) -join ' ')
    $startedAt = [DateTime]::UtcNow
    $child = Start-Process -FilePath $Path -ArgumentList $argumentLine -PassThru -NoNewWindow `
        -RedirectStandardOutput $stdoutPath -RedirectStandardError $stderrPath
    $operationFailure = $null
    try {
        $child.Refresh()
        $childStartTicks = $child.StartTime.ToUniversalTime().Ticks
        [pscustomobject]@{
            operation = $Operation
            executable = $Path
            arguments = $Arguments
            pid = $child.Id
            start_time_utc = $child.StartTime.ToUniversalTime().ToString('o')
            timeout_seconds = $TimeoutSeconds
        } | ConvertTo-Json | Set-Content -LiteralPath $processPath -Encoding utf8
        if (-not $child.WaitForExit($TimeoutSeconds * 1000)) {
            [pscustomobject]@{
                operation = $Operation
                pid = $child.Id
                started_at_utc = $startedAt.ToString('o')
                timeout_seconds = $TimeoutSeconds
                stdout_path = $stdoutPath
                stderr_path = $stderrPath
                partial_output_preserved = $true
            } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $OutputDirectory "$Operation.timeout.json") -Encoding utf8
            throw "$Operation exceeded its $TimeoutSeconds second deadline; partial stdout/stderr were retained."
        }
        $child.Refresh()
        if (-not $child.HasExited -or $child.StartTime.ToUniversalTime().Ticks -ne $childStartTicks) {
            throw "$Operation process identity changed before its exit code was recorded."
        }
        $exitCode = $child.ExitCode
        $exitCode | Set-Content -LiteralPath $exitPath -Encoding ascii
    }
    catch {
        $operationFailure = $_.Exception.Message
    }
    finally {
        try {
            $child.Refresh()
            if (-not $child.HasExited) {
                $child.Kill()
                if (-not $child.WaitForExit(5000)) { throw "$Operation owned process did not stop within 5 seconds." }
            }
        }
        catch {
            if ($null -eq $operationFailure) { $operationFailure = $_.Exception.Message }
            else { $operationFailure += "; owned-process cleanup: $($_.Exception.Message)" }
        }
    }
    if ($null -ne $operationFailure) { throw $operationFailure }
    return [pscustomobject]@{
        ExitCode = $exitCode
        StdoutPath = $stdoutPath
        StderrPath = $stderrPath
        Text = ((Read-BoundedText -Path $stdoutPath) + "`n" + (Read-BoundedText -Path $stderrPath))
    }
}

function Assert-NoActiveWprRecording {
    param([Parameter(Mandatory = $true)][string]$Text)
    $noRecording = $Text -match '(?im)^\s*WPR is not recording\.?\s*$' -or
        $Text -match '(?im)^\s*Status\s*:\s*Not Recording\s*$' -or
        $Text -match '(?i)there is no trace recording in progress|no recording is in progress|no active recording'
    if (-not $noRecording) {
        throw 'WPR status did not explicitly establish that no recording is active; diagnostic recording was not started.'
    }
}

function Assert-PackagedProcessIdentity {
    param(
        [Parameter(Mandatory = $true)][System.Diagnostics.Process]$ExpectedProcess,
        [Parameter(Mandatory = $true)][string]$ExpectedPath,
        [Parameter(Mandatory = $true)][string]$ExpectedSha256,
        [Parameter(Mandatory = $true)][long]$ExpectedStartTimeTicks
    )
    $ExpectedProcess.Refresh()
    if ($ExpectedProcess.HasExited) { return $false }
    $actual = Get-Process -Id $ExpectedProcess.Id -ErrorAction Stop
    if ([string]::IsNullOrWhiteSpace($actual.Path)) { throw 'Cannot determine diagnostic process executable path.' }
    $actualPath = (Resolve-Path -LiteralPath $actual.Path -ErrorAction Stop).ProviderPath
    if (-not [string]::Equals($actualPath, $ExpectedPath, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Diagnostic process executable path changed; refusing to stop an unverified PID.'
    }
    if ($actual.StartTime.ToUniversalTime().Ticks -ne $ExpectedStartTimeTicks) {
        throw 'Diagnostic process start time changed; refusing to stop a reused PID.'
    }
    $actualSha256 = (Get-FileHash -LiteralPath $actualPath -Algorithm SHA256 -ErrorAction Stop).Hash
    if ($actualSha256 -ne $ExpectedSha256) {
        throw 'Diagnostic process executable hash changed; refusing to stop an unverified PID.'
    }
    return $true
}

try {
    $resolvedExecutable = (Resolve-Path -LiteralPath $ExecutablePath -ErrorAction Stop).ProviderPath
    if (-not (Test-Path -LiteralPath $resolvedExecutable -PathType Leaf)) {
        throw 'Packaged Windows executable is missing.'
    }
    $diagnostics = (Resolve-Path -LiteralPath $DiagnosticsDirectory -ErrorAction Stop).ProviderPath
    if (-not (Test-Path -LiteralPath $diagnostics -PathType Container)) {
        throw 'Windows diagnostics directory is missing.'
    }
    $expectedSha256 = (Get-FileHash -LiteralPath $resolvedExecutable -Algorithm SHA256 -ErrorAction Stop).Hash
    $wprCommand = Get-Command -Name 'wpr.exe' -ErrorAction SilentlyContinue
    $tracerptCommand = Get-Command -Name 'tracerpt.exe' -ErrorAction SilentlyContinue
    [pscustomobject]@{
        wpr = $(if ($null -eq $wprCommand) { $null } else { $wprCommand.Source })
        tracerpt = $(if ($null -eq $tracerptCommand) { $null } else { $tracerptCommand.Source })
        powershell = $PSVersionTable.PSVersion.ToString()
    } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $diagnostics 'trace-tools.json') -Encoding utf8
    if ($null -eq $wprCommand) { throw 'wpr.exe is unavailable; Windows startup diagnosis was not run.' }

    $wprFileVersion = [System.Diagnostics.FileVersionInfo]::GetVersionInfo($wprCommand.Source).FileVersion
    [pscustomobject]@{ path = $wprCommand.Source; file_version = $wprFileVersion } |
        ConvertTo-Json | Set-Content -LiteralPath (Join-Path $diagnostics 'wpr-file-version.json') -Encoding utf8
    $help = Invoke-BoundedNative -Path $wprCommand.Source -Arguments @('-help', 'advanced') `
        -Operation 'wpr-help-advanced' -OutputDirectory $diagnostics -TimeoutSeconds 30
    if ($help.ExitCode -ne 0 -or $help.Text -notmatch '(?i)-instancename') {
        throw 'WPR advanced help did not confirm -instancename support; no default-instance fallback is allowed.'
    }
    $profiles = Invoke-BoundedNative -Path $wprCommand.Source -Arguments @('-profiles') `
        -Operation 'wpr-profiles' -OutputDirectory $diagnostics -TimeoutSeconds 30
    if ($profiles.ExitCode -ne 0) { throw "wpr -profiles failed with exit code $($profiles.ExitCode)." }
    if ($profiles.Text -notmatch '(?im)^\s*GeneralProfile(?:\s|$)') {
        throw 'GeneralProfile is not listed by wpr -profiles; no alternate profile was selected.'
    }
    $beforeStatus = Invoke-BoundedNative -Path $wprCommand.Source -Arguments @('-status') `
        -Operation 'wpr-status-before' -OutputDirectory $diagnostics -TimeoutSeconds 30
    if ($beforeStatus.ExitCode -ne 0) { throw "wpr -status failed with exit code $($beforeStatus.ExitCode)." }
    Assert-NoActiveWprRecording -Text $beforeStatus.Text

    $wprInstance = "KatanAStartupDiag-$([guid]::NewGuid().ToString('N'))"
    $etl = Join-Path $diagnostics 'windows-startup-diagnostic.etl'
    [pscustomobject]@{
        instance_name = $wprInstance
        profile = 'GeneralProfile'
        etl_path = $etl
        start_attempted = $true
        ownership_scope = 'unique instance name only; never stops the default WPR instance'
        recorded_at_utc = [DateTime]::UtcNow.ToString('o')
    } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $diagnostics 'wpr-ownership.json') -Encoding utf8
    $wprStartAttempted = $true
    $start = Invoke-BoundedNative -Path $wprCommand.Source `
        -Arguments @('-start', 'GeneralProfile', '-instancename', $wprInstance) `
        -Operation 'wpr-start' -OutputDirectory $diagnostics -TimeoutSeconds 30
    if ($start.ExitCode -ne 0) { throw "wpr -start GeneralProfile failed with exit code $($start.ExitCode)." }
    $ownedStatus = Invoke-BoundedNative -Path $wprCommand.Source `
        -Arguments @('-status', '-instancename', $wprInstance) `
        -Operation 'wpr-status-owned' -OutputDirectory $diagnostics -TimeoutSeconds 30
    if ($ownedStatus.ExitCode -ne 0) { throw "WPR owned-instance status failed with exit code $($ownedStatus.ExitCode)." }
    if ($ownedStatus.Text -notmatch '(?im)^\s*WPR recording is in progress\.{0,3}\s*$' -and
        $ownedStatus.Text -notmatch '(?im)^\s*WPR is recording\.?\s*$' -and
        $ownedStatus.Text -notmatch '(?im)^\s*Status\s*:\s*Recording\s*$') {
        throw 'WPR status did not confirm the unique diagnostic instance is recording.'
    }
    $collectorsBefore = Invoke-BoundedNative -Path $wprCommand.Source `
        -Arguments @('-status', 'collectors', '-details', '-instancename', $wprInstance) `
        -Operation 'wpr-collectors-before' -OutputDirectory $diagnostics -TimeoutSeconds 30
    if ($collectorsBefore.ExitCode -ne 0) { throw "Collector status failed before capture: $($collectorsBefore.ExitCode)." }

    $nonce = [guid]::NewGuid().ToString('N')
    $config = Join-Path $diagnostics "empty-config-$nonce"
    New-Item -ItemType Directory -Path $config -ErrorAction Stop | Out-Null
    $heartbeat = Join-Path $diagnostics "heartbeat-$nonce.txt"
    $stdout = Join-Path $diagnostics 'diagnostic-stdout.log'
    $stderr = Join-Path $diagnostics 'diagnostic-stderr.log'
    $oldConfig = $env:KATANA_CONFIG_DIR
    $oldHeartbeat = $env:KATANA_STARTUP_HEARTBEAT
    $oldDebug = $env:DEBUG
    $oldRustLog = $env:RUST_LOG
    try {
        $env:KATANA_CONFIG_DIR = $config
        $env:KATANA_STARTUP_HEARTBEAT = $heartbeat
        $env:DEBUG = 'true'
        $env:RUST_LOG = 'warn,katana_ui=info,katana_core=info,KatanA=info,eframe::native::run=trace,eframe::native::wgpu_integration=trace,egui_wgpu=debug,winit::platform_impl::windows::event_loop=trace'
        $process = Start-Process -FilePath $resolvedExecutable `
            -WorkingDirectory (Split-Path -Parent $resolvedExecutable) -PassThru `
            -RedirectStandardOutput $stdout -RedirectStandardError $stderr
    }
    finally {
        $env:KATANA_CONFIG_DIR = $oldConfig
        $env:KATANA_STARTUP_HEARTBEAT = $oldHeartbeat
        $env:DEBUG = $oldDebug
        $env:RUST_LOG = $oldRustLog
    }
    $process.Refresh()
    $processStartTicks = $process.StartTime.ToUniversalTime().Ticks
    [pscustomobject]@{
        executable = $resolvedExecutable
        sha256 = $expectedSha256
        pid = $process.Id
        start_time_utc = $process.StartTime.ToUniversalTime().ToString('o')
        isolated_config = $config
        heartbeat = $heartbeat
        capture_limit_seconds = 10
        stdout_path = $stdout
        stderr_path = $stderr
        note = 'Diagnostic observation only; original packaged smoke result remains authoritative.'
    } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $diagnostics 'diagnostic-process.json') -Encoding utf8

    $timer = [System.Diagnostics.Stopwatch]::StartNew()
    while ($timer.Elapsed.TotalSeconds -lt 10) {
        if ($process.WaitForExit(200)) { break }
    }
    $timer.Stop()
    $process.Refresh()
    [pscustomobject]@{
        pid = $process.Id
        elapsed_seconds = [Math]::Round($timer.Elapsed.TotalSeconds, 3)
        exited_within_capture = $process.HasExited
        exit_code = $(if ($process.HasExited) { $process.ExitCode } else { $null })
        heartbeat_present = Test-Path -LiteralPath $heartbeat -PathType Leaf
        interpretation = 'Diagnostic observation only; this result does not pass or replace the original smoke.'
    } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $diagnostics 'diagnostic-observation.json') -Encoding utf8
}
catch {
    $failure = $_.Exception.Message
    if ($null -ne $DiagnosticsDirectory -and (Test-Path -LiteralPath $DiagnosticsDirectory -PathType Container)) {
        $failure | Set-Content -LiteralPath (Join-Path $DiagnosticsDirectory 'diagnostic-error.txt') -Encoding utf8 -ErrorAction Continue
    }
}
finally {
    if ($null -ne $process) {
        try {
            $process.Refresh()
            if (-not $process.HasExited) {
                $stillOwned = Assert-PackagedProcessIdentity -ExpectedProcess $process `
                    -ExpectedPath $resolvedExecutable -ExpectedSha256 $expectedSha256 `
                    -ExpectedStartTimeTicks $processStartTicks
                if ($stillOwned) {
                    $process.Kill()
                    if (-not $process.WaitForExit(5000)) { throw 'Owned diagnostic process did not exit after stop request.' }
                }
            }
        }
        catch {
            if ($null -eq $failure) { $failure = $_.Exception.Message }
        }
    }
    if ($wprStartAttempted) {
        try {
            $collectorsAfter = Invoke-BoundedNative -Path $wprCommand.Source `
                -Arguments @('-status', 'collectors', '-details', '-instancename', $wprInstance) `
                -Operation 'wpr-collectors-after' -OutputDirectory $DiagnosticsDirectory -TimeoutSeconds 30
            if ($collectorsAfter.ExitCode -ne 0) { throw "Collector status failed after capture: $($collectorsAfter.ExitCode)." }
        }
        catch {
            if ($null -eq $failure) { $failure = $_.Exception.Message }
        }
        try {
            $stop = Invoke-BoundedNative -Path $wprCommand.Source `
                -Arguments @('-stop', $etl, 'bounded packaged startup diagnosis', '-skipPdbGen', '-instancename', $wprInstance) `
                -Operation 'wpr-stop' -OutputDirectory $DiagnosticsDirectory -TimeoutSeconds 60
            if ($stop.ExitCode -ne 0) {
                $cleanupMessage = "WPR same-instance cleanup returned $($stop.ExitCode); no default/global session was touched."
                $cleanupMessage | Set-Content -LiteralPath (Join-Path $DiagnosticsDirectory 'wpr-cleanup-error.txt') -Encoding utf8
                if ($null -eq $failure) { $failure = $cleanupMessage }
            }
            if ($stop.ExitCode -eq 0 -and $null -ne $tracerptCommand -and (Test-Path -LiteralPath $etl -PathType Leaf)) {
                $conversion = Invoke-BoundedNative -Path $tracerptCommand.Source `
                    -Arguments @($etl, '-o', (Join-Path $DiagnosticsDirectory 'windows-startup-diagnostic.csv'), '-of', 'CSV', '-y') `
                    -Operation 'tracerpt-conversion' -OutputDirectory $DiagnosticsDirectory -TimeoutSeconds 60
                if ($conversion.ExitCode -ne 0) {
                    $conversionMessage = "tracerpt conversion returned $($conversion.ExitCode); ETL interpretation remains unavailable."
                    $conversionMessage | Set-Content -LiteralPath (Join-Path $DiagnosticsDirectory 'tracerpt-error.txt') -Encoding utf8
                    if ($null -eq $failure) { $failure = $conversionMessage }
                }
            }
        }
        catch {
            $cleanupMessage = $_.Exception.Message
            if (Test-Path -LiteralPath $DiagnosticsDirectory -PathType Container) {
                $cleanupMessage | Set-Content -LiteralPath (Join-Path $DiagnosticsDirectory 'wpr-cleanup-error.txt') -Encoding utf8 -ErrorAction Continue
            }
            if ($null -eq $failure) { $failure = $cleanupMessage }
        }
    }
}

if ($null -ne $failure) { throw "Windows startup diagnosis failed: $failure" }
Write-Host "Windows startup diagnosis retained in $DiagnosticsDirectory; original smoke result is unchanged."
