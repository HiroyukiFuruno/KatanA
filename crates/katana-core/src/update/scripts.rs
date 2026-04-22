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
                 * Start-Sleep extended to 2 s to allow the parent process to fully release
                 * file locks before the rename attempt. */
                r#"$ProgressPreference = 'SilentlyContinue';
Start-Sleep -s 2;
$target = '{target}';
$bak = '{target}.bak';
$extracted = '{extracted}';
if (Test-Path $bak) {{ Remove-Item -Force $bak -ErrorAction SilentlyContinue }};
if (Test-Path $target) {{ Move-Item -Force $target $bak -ErrorAction SilentlyContinue }};
Move-Item -Force $extracted $target -ErrorAction SilentlyContinue;
if (Test-Path $target) {{
    Start-Process -WindowStyle Hidden $target;
}} else {{
    if (Test-Path $bak) {{ Move-Item -Force $bak $target -ErrorAction SilentlyContinue }};
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
