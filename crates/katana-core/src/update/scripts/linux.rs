use std::path::Path;

pub(super) fn generate_script_content(
    target_app: &Path,
    extracted_app: &Path,
    temp_dir_path: &Path,
) -> String {
    let target_sidecar = target_app
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join("kdv-office-worker");
    let extracted_sidecar = extracted_app
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join("kdv-office-worker");
    format!(
        /* WHY: Preserve executable permissions and update the app and worker as one
         * transaction so a failure restores both previous versions. */
        r#"#!/bin/bash
sleep 2
TARGET_BAK="{target}.bak"
TARGET_SIDECAR="{target_sidecar}"
SIDECAR_BAK="{target_sidecar}.bak"
EXTRACTED_SIDECAR="{extracted_sidecar}"
TARGET_BACKED_UP=0
SIDECAR_BACKED_UP=0

cleanup() {{
    rm -rf "{temp_dir}"
}}

rollback() {{
    rm -f "{target}"
    if [ "$TARGET_BACKED_UP" -eq 1 ]; then
        mv "$TARGET_BAK" "{target}" || return 1
    fi
    rm -f "$TARGET_SIDECAR"
    if [ "$SIDECAR_BACKED_UP" -eq 1 ]; then
        mv "$SIDECAR_BAK" "$TARGET_SIDECAR" || return 1
    fi
}}

rm -f "$TARGET_BAK"
rm -f "$SIDECAR_BAK"
if [ -f "{target}" ]; then
    if ! mv "{target}" "$TARGET_BAK"; then
        cleanup
        exit 1
    fi
    TARGET_BACKED_UP=1
fi
if [ -f "$TARGET_SIDECAR" ]; then
    if ! mv "$TARGET_SIDECAR" "$SIDECAR_BAK"; then
        if [ "$TARGET_BACKED_UP" -eq 1 ]; then
            mv "$TARGET_BAK" "{target}" || true
        fi
        cleanup
        exit 1
    fi
    SIDECAR_BACKED_UP=1
fi
if cp "$EXTRACTED_SIDECAR" "$TARGET_SIDECAR" &&
   chmod +x "$TARGET_SIDECAR" &&
   mv "{extracted}" "{target}" &&
   chmod +x "{target}"; then
    "{target}" &
    rm -f "$TARGET_BAK"
    rm -f "$SIDECAR_BAK"
    cleanup
else
    rollback || true
    cleanup
    exit 1
fi
"#,
        target = target_app.display(),
        extracted = extracted_app.display(),
        temp_dir = temp_dir_path.display(),
        target_sidecar = target_sidecar.display(),
        extracted_sidecar = extracted_sidecar.display()
    )
}
