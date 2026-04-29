use std::path::Path;

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
            format!(
                /* WHY: Bug 2 fix — Verify the new binary is in place with Test-Path after
                 * Move-Item (more reliable than $? which reflects the last command's result).
                 * Bug 3 fix — Use -WindowStyle Hidden when launching the new process so the
                 * restarted KatanA does not briefly show a console window.
                 * Add retry loop to handle file lock release delays. */
                r#"param($parentPid);
$ProgressPreference = 'SilentlyContinue';
if ($parentPid) {{
    Wait-Process -Id $parentPid -Timeout 30 -ErrorAction SilentlyContinue;
}}
$target = '{target}';
$bak = '{target}.bak';
$extracted = '{extracted}';
if (Test-Path $bak) {{ Remove-Item -Force $bak -ErrorAction SilentlyContinue }};
if (Test-Path $target) {{ Move-Item -Force $target $bak -ErrorAction SilentlyContinue }};

$retryCount = 0;
$success = $false;
while ($retryCount -lt 5) {{
    Move-Item -Force $extracted $target -ErrorAction SilentlyContinue;
    if (Test-Path $target) {{
        $success = $true;
        break;
    }}
    $retryCount++;
    Start-Sleep -s 1;
}}

if ($success) {{
    Start-Process -WindowStyle Hidden $target;
}} else {{
    if (Test-Path $bak) {{ Move-Item -Force $bak $target -ErrorAction SilentlyContinue }};
    Add-Type -AssemblyName PresentationFramework;
    [System.Windows.MessageBox]::Show('Could not complete the application update. The original version has been restored.', 'Update Failed', 'OK', 'Error');
}}
Remove-Item -Recurse -Force '{temp_dir}' -ErrorAction SilentlyContinue;
"#,
                target = target_app.display(),
                extracted = extracted_app.display(),
                temp_dir = temp_dir_path.display()
            )
        }

        #[cfg(target_os = "linux")]
        {
            format!(
                /* WHY: Bug 2 fix — chmod +x ensures the extracted binary has execute permission
                 * before being launched (extracted files may not inherit the original permissions).
                 * set -e removed to allow explicit if/else error handling.
                 * Sleep extended to 2 s to ensure the parent process exits before the rename. */
                r#"#!/bin/bash
sleep 2
TARGET_BAK="{target}.bak"
rm -f "$TARGET_BAK"
if [ -f "{target}" ]; then
    mv "{target}" "$TARGET_BAK"
fi
if mv "{extracted}" "{target}"; then
    chmod +x "{target}"
    "{target}" &
    rm -f "$TARGET_BAK"
    rm -rf "{temp_dir}"
else
    if [ -f "$TARGET_BAK" ]; then
        mv "$TARGET_BAK" "{target}"
    fi
    rm -rf "{temp_dir}"
    exit 1
fi
"#,
                target = target_app.display(),
                extracted = extracted_app.display(),
                temp_dir = temp_dir_path.display()
            )
        }
    }
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
            assert!(content.contains("param($parentPid);"));
            assert!(content.contains("Wait-Process -Id $parentPid"));
            assert!(content.contains("$retryCount = 0;"));
            assert!(content.contains("while ($retryCount -lt 5)"));
            assert!(content.contains("Add-Type -AssemblyName PresentationFramework"));
        }

        #[cfg(target_os = "linux")]
        {
            assert!(content.contains("mv \"extracted_app\" \"target_app\""));
        }
    }
}
