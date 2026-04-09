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
                r#"@echo off
timeout /t 2 /nobreak >nul
set TARGET_BAK={target}.bak
if exist "%TARGET_BAK%" del /f /q "%TARGET_BAK%"
if exist "{target}" move /y "{target}" "%TARGET_BAK%"
move /y "{extracted}" "{target}"
if errorlevel 1 (
    if exist "%TARGET_BAK%" move /y "%TARGET_BAK%" "{target}"
    echo Update failed.
) else (
    start "" "{target}"
)
rd /s /q "{temp_dir}"
del "%~f0"
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
