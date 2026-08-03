use std::path::Path;

#[cfg(target_os = "linux")]
mod linux;

pub(crate) struct UpdateScriptOps;

impl UpdateScriptOps {
    pub(crate) fn generate_script_content(
        target_app: &Path,
        extracted_app: &Path,
        temp_dir_path: &Path,
    ) -> String {
        #[cfg(target_os = "macos")]
        {
            format!(
                r#"#!/bin/bash
set -e
sleep 1
TARGET_BAK="{target}.bak"
rm -rf "$TARGET_BAK"

if [ -d "{target}" ]; then
    echo "Backing up existing installation..."
    mv "{target}" "$TARGET_BAK"
fi

if ! mv "{extracted}" "{target}"; then
    echo "Swap failed! Rolling back..."
    osascript -e 'display alert "Update Failed" message "Could not complete the application update. The original version has been restored." as critical' || true
    rm -rf "{target}"

    if [ -d "$TARGET_BAK" ]; then
        mv "$TARGET_BAK" "{target}"
    fi

    open "{target}" || true
    rm -rf "{temp_dir}"
    exit 1
fi

xattr -cr "{target}" || true
open "{target}"
rm -rf "$TARGET_BAK"
rm -rf "{temp_dir}"
"#,
                target = target_app.display(),
                extracted = extracted_app.display(),
                temp_dir = temp_dir_path.display()
            )
        }

        #[cfg(target_os = "windows")]
        {
            let target_esc = escape_powershell_single_quoted_path(target_app);
            let extracted_esc = escape_powershell_single_quoted_path(extracted_app);
            let temp_dir_esc = escape_powershell_single_quoted_path(temp_dir_path);
            let target_sidecar_esc = escape_powershell_single_quoted_path(
                &target_app
                    .parent()
                    .unwrap_or_else(|| Path::new(""))
                    .join("kdv-office-worker.exe"),
            );
            let extracted_sidecar_esc = escape_powershell_single_quoted_path(
                &extracted_app
                    .parent()
                    .unwrap_or_else(|| Path::new(""))
                    .join("kdv-office-worker.exe"),
            );

            format!(
                r#"param($parentPid);
$ErrorActionPreference = 'SilentlyContinue';
$ProgressPreference = 'SilentlyContinue';
$target = '{target}';
$bak = '{target}.bak';
$extracted = '{extracted}';
$targetSidecar = '{target_sidecar}';
$sidecarBak = '{target_sidecar}.bak';
$extractedSidecar = '{extracted_sidecar}';
$logDir = Join-Path $env:LOCALAPPDATA 'KatanA';
$logPath = Join-Path $logDir 'update.log';

function Write-UpdateLog($phase, $result, $reason) {{
    if (-not (Test-Path $logDir)) {{ New-Item -ItemType Directory -Force -Path $logDir | Out-Null }}
    Add-Content -Path $logPath -Value ((Get-Date -Format o), $phase, $result, $target, $reason -join ' ');
}}

if ($parentPid -as [int]) {{
    Wait-Process -Id ([int]$parentPid) -Timeout 30 -ErrorAction SilentlyContinue;
}}

if (Test-Path $bak) {{ Remove-Item -Force $bak -ErrorAction SilentlyContinue }};
if (Test-Path $sidecarBak) {{ Remove-Item -Force $sidecarBak -ErrorAction SilentlyContinue }};

$success = $false;
for ($retryCount = 0; $retryCount -lt 30; $retryCount++) {{
    try {{
        if ((Test-Path $target) -and (-not (Test-Path $bak))) {{
            Move-Item -Force $target $bak -ErrorAction Stop;
        }}
        if ((Test-Path $targetSidecar) -and (-not (Test-Path $sidecarBak))) {{
            Move-Item -Force $targetSidecar $sidecarBak -ErrorAction Stop;
        }}
        Copy-Item -Force $extractedSidecar $targetSidecar -ErrorAction Stop;
        Move-Item -Force $extracted $target -ErrorAction Stop;
        $success = $true;
        Write-UpdateLog 'update' 'ok' "retry=$retryCount";
        break;
    }} catch {{
        Write-UpdateLog 'update' 'retry' "retry=$retryCount error=$($_.Exception.Message)";
    }}
    Start-Sleep -s 1;
}}

if ($success) {{
    Remove-Item -Force $bak -ErrorAction SilentlyContinue;
    Remove-Item -Force $sidecarBak -ErrorAction SilentlyContinue;
    Start-Process $target -WorkingDirectory (Split-Path $target);
    Write-UpdateLog 'launch' 'ok' '';
}} else {{
    if (Test-Path $bak) {{
        Move-Item -Force $bak $target -ErrorAction SilentlyContinue;
    }}
    Remove-Item -Force $targetSidecar -ErrorAction SilentlyContinue;
    if (Test-Path $sidecarBak) {{
        Move-Item -Force $sidecarBak $targetSidecar -ErrorAction SilentlyContinue;
    }}
    Write-UpdateLog 'rollback' 'done' '';
    Add-Type -AssemblyName PresentationFramework;
    $msg = "Could not complete the application update. The original version has been restored.`n`nDetails: $logPath";
    [System.Windows.MessageBox]::Show($msg, 'Update Failed', 'OK', 'Error') | Out-Null;
}}

# Best effort cleanup
Start-Sleep -s 2;
Remove-Item -Recurse -Force '{temp_dir}' -ErrorAction SilentlyContinue;
"#,
                target = target_esc,
                extracted = extracted_esc,
                temp_dir = temp_dir_esc,
                target_sidecar = target_sidecar_esc,
                extracted_sidecar = extracted_sidecar_esc
            )
        }

        #[cfg(target_os = "linux")]
        {
            linux::generate_script_content(target_app, extracted_app, temp_dir_path)
        }
    }
}

#[cfg(any(target_os = "windows", test))]
fn escape_powershell_single_quoted_path(path: &Path) -> String {
    path.display().to_string().replace("'", "''")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_generate_script_content() {
        let content = UpdateScriptOps::generate_script_content(
            Path::new("target_app"),
            Path::new("extracted_app"),
            Path::new("temp_dir"),
        );

        #[cfg(target_os = "macos")]
        {
            assert!(content.contains("mv \"extracted_app\" \"target_app\""));
        }

        #[cfg(target_os = "windows")]
        {
            assert!(content.contains("function Write-UpdateLog"));
            assert!(content.contains("Move-Item -Force $target $bak -ErrorAction Stop;"));
            assert!(content.contains("update.log"));
            assert!(content.contains("Add-Type -AssemblyName PresentationFramework"));
            assert!(content.contains("kdv-office-worker.exe"));
            assert!(content.contains("Copy-Item -Force $extractedSidecar $targetSidecar"));
        }

        #[cfg(target_os = "linux")]
        {
            assert!(content.contains("mv \"extracted_app\" \"target_app\""));
            assert!(content.contains("EXTRACTED_SIDECAR=\"kdv-office-worker\""));
            assert!(content.contains("TARGET_SIDECAR=\"kdv-office-worker\""));
            assert!(content.contains("cp \"$EXTRACTED_SIDECAR\" \"$TARGET_SIDECAR\""));
            assert!(content.contains("rollback()"));
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn linux_script_replaces_target_even_when_asset_name_differs() {
        let content = UpdateScriptOps::generate_script_content(
            Path::new("/home/linuxbrew/.linuxbrew/bin/katana-desktop"),
            Path::new("/tmp/katana-update/extracted/KatanA"),
            Path::new("/tmp/katana-update"),
        );

        assert!(content.contains(
            "mv \"/tmp/katana-update/extracted/KatanA\" \"/home/linuxbrew/.linuxbrew/bin/katana-desktop\""
        ));
        assert!(content.contains("chmod +x \"/home/linuxbrew/.linuxbrew/bin/katana-desktop\""));
        assert!(
            content
                .contains("EXTRACTED_SIDECAR=\"/tmp/katana-update/extracted/kdv-office-worker\"")
        );
        assert!(
            content.contains("TARGET_SIDECAR=\"/home/linuxbrew/.linuxbrew/bin/kdv-office-worker\"")
        );
        assert!(content.contains("cp \"$EXTRACTED_SIDECAR\" \"$TARGET_SIDECAR\""));
        assert!(content.contains("chmod +x \"$TARGET_SIDECAR\""));
        assert!(content.contains("\"/home/linuxbrew/.linuxbrew/bin/katana-desktop\" &"));
    }

    #[test]
    fn escape_powershell_single_quoted_path_doubles_single_quotes() {
        let escaped =
            escape_powershell_single_quoted_path(Path::new(r"C:\Users\O'Connor\AppData\Temp"));

        assert_eq!(escaped, r"C:\Users\O''Connor\AppData\Temp");
    }
}
