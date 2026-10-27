def assert_graphics_profile_contract(source):
    required = (
        "if ($WaitStacks) {\n        $gpuProfilePath",
        "@('-exportprofile', $profile.name, $profile.path)",
        "-Operation $profile.operation -OutputDirectory $diagnostics -TimeoutSeconds 30",
        "GPU.Verbose.wprp",
        "DesktopComposition.Verbose.wprp",
        "Get-FileHash -LiteralPath $profile.path -Algorithm SHA256",
        "wpr_file_version = $wprFileVersion",
        "requested_profile = 'KatanAWaitStacks.Verbose.Memory'",
        "profile_path = $wprProfilePath",
        "profile_sha256 = $wprProfileSha256",
        "interpretation = $null",
        "requested_profiles = @('KatanAWaitStacks.Verbose.Memory', 'GPU.Verbose.Memory', 'DesktopComposition.Verbose.Memory')",
        "$graphicsStatusMatches.Count -ne 3",
        "$graphicsStatusMatches = @(\n            @(",
        "no alternate profile was selected",
        "@('-start', 'GeneralProfile', '-instancename', $wprInstance)",
    )
    missing = [item for item in required if item not in source]
    if missing:
        raise AssertionError(f"Missing graphics-profile contract text: {missing}")
    for profile in ("KatanAWaitStacks\\.Verbose\\.Memory", "GPU\\.Verbose\\.Memory", "DesktopComposition\\.Verbose\\.Memory"):
        if profile not in source:
            raise AssertionError(f"Missing memory-mode status requirement: {profile}")
    if "-filemode" in source.lower():
        raise AssertionError("Graphics-profile trace must keep the existing memory-mode configuration.")
