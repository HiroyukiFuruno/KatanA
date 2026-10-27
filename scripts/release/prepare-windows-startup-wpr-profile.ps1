[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$DiagnosticsDirectory
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$profileName = 'KatanAWaitStacks'
$sourcePath = Join-Path $PSScriptRoot 'windows-startup-wait-stacks.wprp'
$destinationPath = Join-Path $DiagnosticsDirectory 'windows-startup-wait-stacks.wprp'
$metadataPath = Join-Path $DiagnosticsDirectory 'wpr-profile-definition.json'

if (-not (Test-Path -LiteralPath $DiagnosticsDirectory -PathType Container)) {
    throw 'WPR profile output directory is missing.'
}
foreach ($path in @($destinationPath, $metadataPath)) {
    if (Test-Path -LiteralPath $path) { throw "Refusing to overwrite WPR profile evidence: $path" }
}
if (-not (Test-Path -LiteralPath $sourcePath -PathType Leaf)) { throw 'Fixed wait-stack WPR profile is missing.' }

# WHY: 診断時に固定profileの定義と実行証跡を同じartifactへ残す。
[xml]$document = Get-Content -LiteralPath $sourcePath -Raw -ErrorAction Stop
$profiles = @($document.SelectNodes("/WindowsPerformanceRecorder/Profiles/Profile[@Name='$profileName']"))
$expectedIds = @("$profileName.Verbose.Memory", "$profileName.Verbose.File")
$actualIds = @($profiles | ForEach-Object { $_.GetAttribute('Id') })
$differentIds = @(Compare-Object $expectedIds $actualIds)
if ($profiles.Count -ne 2 -or $differentIds.Count -ne 0) {
    throw 'Wait-stack WPR profile must define its fixed memory and file variants.'
}
$collector = $document.SelectSingleNode("/WindowsPerformanceRecorder/Profiles/SystemCollector[@Id='KatanAWaitStacksCollector']")
$provider = $document.SelectSingleNode("/WindowsPerformanceRecorder/Profiles/SystemProvider[@Id='KatanAWaitStacksProvider']")
if ($null -eq $collector -or $collector.GetAttribute('Name') -ne 'NT Kernel Logger') {
    throw 'Wait-stack WPR profile must define its fixed NT Kernel Logger collector.'
}
if ($null -eq $provider) { throw 'Wait-stack WPR system provider is missing.' }
foreach ($profile in $profiles) {
    $collectorReference = $profile.SelectSingleNode("./Collectors/SystemCollectorId[@Value='KatanAWaitStacksCollector']")
    if ($null -eq $collectorReference) { throw 'Wait-stack WPR profile collector reference is missing.' }
    $providerReference = $collectorReference.SelectSingleNode("./SystemProviderId[@Value='KatanAWaitStacksProvider']")
    if ($null -eq $providerReference) { throw 'Wait-stack WPR profile collector reference is incomplete.' }
}
$keywords = @($provider.SelectNodes('./Keywords/Keyword') | ForEach-Object { $_.GetAttribute('Value') })
$stacks = @($provider.SelectNodes('./Stacks/Stack') | ForEach-Object { $_.GetAttribute('Value') })
foreach ($required in @('ProcessThread', 'Loader', 'CSwitch', 'ReadyThread')) {
    if ($keywords -notcontains $required) { throw "Required system keyword is missing: $required" }
}
foreach ($required in @('CSwitch', 'ReadyThread')) {
    if ($stacks -notcontains $required) { throw "Required system stack is missing: $required" }
}
Copy-Item -LiteralPath $sourcePath -Destination $destinationPath -ErrorAction Stop
$sourceHash = (Get-FileHash -LiteralPath $sourcePath -Algorithm SHA256 -ErrorAction Stop).Hash
$copiedHash = (Get-FileHash -LiteralPath $destinationPath -Algorithm SHA256 -ErrorAction Stop).Hash
if ($sourceHash -ne $copiedHash) { throw 'Copied WPR profile hash differs from its fixed source.' }
[pscustomobject]@{
    profile_name = $profileName
    profile_ids = $actualIds
    keywords = $keywords
    stacks = $stacks
    source_path = $sourcePath
    profile_path = $destinationPath
    sha256 = $copiedHash
    schema_reference = 'https://learn.microsoft.com/en-us/windows-hardware/test/wpt/wprcontrolprofiles-schema'
} | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $metadataPath -Encoding utf8
[pscustomobject]@{ profile_name = $profileName; profile_path = $destinationPath; sha256 = $copiedHash }
