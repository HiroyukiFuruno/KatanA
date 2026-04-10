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
                r#"$ProgressPreference = 'SilentlyContinue';
Start-Sleep -s 1;
$target = '{target}';
$bak = '{target}.bak';
$extracted = '{extracted}';
if (Test-Path $bak) {{ Remove-Item -Force $bak -ErrorAction SilentlyContinue }};
if (Test-Path $target) {{ Move-Item -Force $target $bak -ErrorAction SilentlyContinue }};
Move-Item -Force $extracted $target -ErrorAction SilentlyContinue;
if ($?) {{
    Start-Process $target;
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
                r#"#!/bin/bash
set -e
sleep 1
TARGET_BAK="{target}.bak"
rm -f "$TARGET_BAK"
if [ -f "{target}" ]; then
    mv "{target}" "$TARGET_BAK"
fi
if ! mv "{extracted}" "{target}"; then
    if [ -f "$TARGET_BAK" ]; then
        mv "$TARGET_BAK" "{target}"
    fi
    "{target}" &
    rm -rf "{temp_dir}"
    exit 1
fi
"{target}" &
rm -f "$TARGET_BAK"
rm -rf "{temp_dir}"
"#,
                target = target_app.display(),
                extracted = extracted_app.display(),
                temp_dir = temp_dir_path.display()
            )
        }
    }
}
