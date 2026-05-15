use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let lock_path = manifest_dir.join("../../Cargo.lock");
    println!("cargo:rerun-if-changed={}", lock_path.display());

    let lock = fs::read_to_string(&lock_path).expect("Cargo.lock should be readable");
    let version = lock_package_version(&lock, "katana-diagram-renderer")
        .expect("katana-diagram-renderer should exist in Cargo.lock");
    println!("cargo:rustc-env=KATANA_DIAGRAM_RENDERER_VERSION={version}");
}

fn lock_package_version(lock: &str, package_name: &str) -> Option<String> {
    for section in lock.split("[[package]]").skip(1) {
        let mut name = None;
        let mut version = None;
        for line in section.lines() {
            if line.starts_with("name = ") {
                name = quoted_value(line);
            }
            if line.starts_with("version = ") {
                version = quoted_value(line);
            }
        }
        if name.as_deref() == Some(package_name) {
            return version;
        }
    }
    None
}

fn quoted_value(line: &str) -> Option<String> {
    line.split_once('"')?
        .1
        .split_once('"')
        .map(|(value, _)| value.to_string())
}

#[cfg(test)]
mod tests {
    use super::lock_package_version;

    #[test]
    fn lock_package_version_reads_target_package() {
        let lock = r#"
[[package]]
name = "other"
version = "1.0.0"

[[package]]
name = "katana-diagram-renderer"
version = "0.1.0"
"#;

        assert_eq!(
            lock_package_version(lock, "katana-diagram-renderer"),
            Some("0.1.0".to_string())
        );
    }
}
