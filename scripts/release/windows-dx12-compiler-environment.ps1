function Enable-WindowsDx12Compiler {
    param(
        [Parameter(Mandatory = $true)][pscustomobject]$Metadata,
        [Parameter(Mandatory = $true)][string]$ControllerRoot
    )
    $compilerBin = [string]$Metadata.dx12_compiler.compiler_bin_path
    $receiptPath = [System.IO.Path]::GetFullPath((Join-Path $compilerBin '..\..\compiler-receipt.json'))
    if (-not (Test-Path -LiteralPath $receiptPath -PathType Leaf)) {
        throw 'Pinned DXC receipt is missing before the compiler case.'
    }
    $actualReceiptSha256 = (Get-FileHash -LiteralPath $receiptPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actualReceiptSha256 -ne $Metadata.dx12_compiler_receipt_sha256) {
        throw 'Pinned DXC receipt changed after diagnostic preparation.'
    }
    $compilerRoot = Split-Path -Parent (Split-Path -Parent $compilerBin)
    $assetVerifier = Join-Path $ControllerRoot 'scripts/release/windows-dx12-compiler-assets.py'
    $verifiedBin = python $assetVerifier verify --receipt $receiptPath --expected-root $compilerRoot
    if ($LASTEXITCODE -ne 0 -or $verifiedBin.Trim() -ne $compilerBin) {
        throw 'Pinned DXC DLL verification failed before the compiler case.'
    }

    $state = [pscustomobject]@{
        had_path = Test-Path -LiteralPath 'Env:PATH'
        path = [Environment]::GetEnvironmentVariable('PATH', 'Process')
        had_compiler = Test-Path -LiteralPath 'Env:WGPU_DX12_COMPILER'
        compiler = [Environment]::GetEnvironmentVariable('WGPU_DX12_COMPILER', 'Process')
    }
    try {
        [Environment]::SetEnvironmentVariable('PATH', "$compilerBin$([IO.Path]::PathSeparator)$($state.path)", 'Process')
        [Environment]::SetEnvironmentVariable('WGPU_DX12_COMPILER', 'DynamicDxc', 'Process')
    } catch {
        Restore-WindowsDx12CompilerEnvironment -State $state
        throw
    }
    return $state
}

function Restore-WindowsDx12CompilerEnvironment {
    param([Parameter(Mandatory = $true)][pscustomobject]$State)
    if ($State.had_path) {
        [Environment]::SetEnvironmentVariable('PATH', $State.path, 'Process')
    } else {
        Remove-Item -LiteralPath 'Env:PATH' -ErrorAction SilentlyContinue
    }
    if ($State.had_compiler) {
        [Environment]::SetEnvironmentVariable('WGPU_DX12_COMPILER', $State.compiler, 'Process')
    } else {
        Remove-Item -LiteralPath 'Env:WGPU_DX12_COMPILER' -ErrorAction SilentlyContinue
    }
}
