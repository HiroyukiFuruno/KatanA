$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Path (Join-Path $env:GITHUB_WORKSPACE 'target/windows-startup-diagnostics') -Force | Out-Null
if ($env:WINDOWS_RUN_ID -notmatch '^[1-9][0-9]*$') { throw 'windows_run_id must be a positive decimal string' }
if ($env:WINDOWS_SOURCE_COMMIT -notmatch '^[0-9a-f]{40}$') { throw 'windows_source_commit must be a lowercase 40-character commit' }
if ($env:WINDOWS_RUN_ATTEMPT -notmatch '^[1-9][0-9]*$') { throw 'windows_run_attempt must be a positive decimal string' }
$runJson = gh api "repos/$env:GITHUB_REPOSITORY/actions/runs/$env:WINDOWS_RUN_ID"
if ($LASTEXITCODE -ne 0) { throw 'Could not read selected GitHub Actions run' }
$run = $runJson | ConvertFrom-Json
if ($run.path -ne '.github/workflows/build-and-release.yml') { throw 'Selected run is not the v0.22.42 release workflow' }
if ($run.repository.full_name -ne $env:GITHUB_REPOSITORY) { throw 'Selected run belongs to another repository' }
if ($run.head_repository.full_name -ne $env:GITHUB_REPOSITORY) { throw 'Forked release runs are not accepted' }
if ([string]$run.run_attempt -ne $env:WINDOWS_RUN_ATTEMPT) { throw 'Selected run attempt differs' }
if ($run.conclusion -ne 'failure') { throw 'Selected source run is not a failed release run' }
if ($run.head_sha -ne $env:WINDOWS_SOURCE_COMMIT) {
    if ($run.event -ne 'pull_request') { throw 'Only a pull_request run may use the merge-commit source relation' }
    # WHY: PR由来の失敗runはPR headを記録するため、payload identityとmerge commitの親でsourceを結ぶ。
    $commitJson = gh api "repos/$env:GITHUB_REPOSITORY/commits/$env:WINDOWS_SOURCE_COMMIT"
    if ($LASTEXITCODE -ne 0) { throw 'Could not read expected artifact source commit' }
    $expectedCommit = $commitJson | ConvertFrom-Json
    $parentShas = @($expectedCommit.parents | ForEach-Object { $_.sha })
    if ($run.head_sha -notin $parentShas) { throw 'Expected payload source is neither run head nor a direct merge commit child' }
}
$jobsJson = gh api "repos/$env:GITHUB_REPOSITORY/actions/runs/$env:WINDOWS_RUN_ID/jobs?per_page=100"
if ($LASTEXITCODE -ne 0) { throw 'Could not read selected run jobs' }
$jobs = ($jobsJson | ConvertFrom-Json).jobs
$build = @($jobs | Where-Object { $_.name -eq 'Build Windows' })
if ($build.Count -ne 1 -or $build[0].conclusion -ne 'failure') { throw 'Selected run does not contain one failed Build Windows job' }
$smoke = @($build[0].steps | Where-Object { $_.name -eq 'Smoke launch packaged Windows app' })
if ($smoke.Count -ne 1 -or $smoke[0].conclusion -ne 'failure') { throw 'Selected Build Windows failure was not the packaged startup smoke' }
Write-Host "validated_failed_smoke_run=$env:WINDOWS_RUN_ID; attempt=$env:WINDOWS_RUN_ATTEMPT; source=$env:WINDOWS_SOURCE_COMMIT"
