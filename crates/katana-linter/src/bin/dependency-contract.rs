use std::io::{self, Read, Write};
use std::process::ExitCode;

use semver::{Version, VersionReq};
use serde::Deserialize;

const MAX_INPUT_BYTES: u64 = 1_048_576;
const MAX_REQUESTS: usize = 4096;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionRequest {
    requirement: String,
    version: String,
}

struct VersionContract;

impl VersionContract {
    fn read(reader: impl Read) -> Result<Vec<VersionRequest>, String> {
        let mut input = Vec::new();
        reader
            .take(MAX_INPUT_BYTES + 1)
            .read_to_end(&mut input)
            .map_err(|error| error.to_string())?;
        if input.len() as u64 > MAX_INPUT_BYTES {
            return Err("version request input exceeds its byte limit".into());
        }
        let requests: Vec<VersionRequest> =
            serde_json::from_slice(&input).map_err(|error| error.to_string())?;
        if requests.len() > MAX_REQUESTS {
            return Err("version request count exceeds its limit".into());
        }
        Ok(requests)
    }

    fn evaluate(request: &VersionRequest) -> Result<bool, String> {
        let requirement =
            VersionReq::parse(&request.requirement).map_err(|error| error.to_string())?;
        let version = Version::parse(&request.version).map_err(|error| error.to_string())?;
        Ok(requirement.matches(&version))
    }

    fn run(reader: impl Read, mut writer: impl Write) -> Result<(), String> {
        let requests = Self::read(reader)?;
        let matches: Vec<bool> = requests
            .iter()
            .map(Self::evaluate)
            .collect::<Result<_, _>>()?;
        serde_json::to_writer(&mut writer, &matches).map_err(|error| error.to_string())?;
        writer.write_all(b"\n").map_err(|error| error.to_string())
    }
}

fn main() -> ExitCode {
    match VersionContract::run(io::stdin().lock(), io::stdout().lock()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("dependency version contract failed: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_caret_and_prerelease_are_not_approximated() {
        let input = br#"[
          {"requirement":"=2.0.0","version":"1.0.0"},
          {"requirement":"0.22.39","version":"0.22.42"},
          {"requirement":"~1.2, >=1.2.3","version":"1.2.9"},
          {"requirement":"1.*","version":"1.9.0-alpha"},
          {"requirement":"^1.0.0-alpha","version":"1.0.0-beta"}
        ]"#;
        let mut output = Vec::new();
        VersionContract::run(input.as_slice(), &mut output).expect("valid request");
        assert_eq!(output, b"[false,true,true,false,true]\n");
    }

    #[test]
    fn invalid_json_and_unknown_fields_do_not_emit_success() {
        for input in [
            b"null".as_slice(),
            br#"[{"requirement":"1","version":"1.0.0","matches":true}]"#,
            br#"[{"requirement":"invalid","version":"1.0.0"}]"#,
            br#"[{"requirement":"1","version":"01.0.0"}]"#,
        ] {
            let mut output = Vec::new();
            assert!(VersionContract::run(input, &mut output).is_err());
            assert!(output.is_empty());
        }
    }

    #[test]
    fn excessive_bytes_are_rejected_before_json_parsing() {
        let input = vec![b' '; MAX_INPUT_BYTES as usize + 1];
        let error = VersionContract::run(input.as_slice(), Vec::new()).expect_err("oversized");
        assert_eq!(error, "version request input exceeds its byte limit");
    }

    #[test]
    fn excessive_requests_do_not_emit_partial_success() {
        let request = r#"{"requirement":"1","version":"1.0.0"}"#;
        let input = format!("[{}]", vec![request; MAX_REQUESTS + 1].join(","));
        let mut output = Vec::new();
        let error = VersionContract::run(input.as_bytes(), &mut output).expect_err("too many");
        assert_eq!(error, "version request count exceeds its limit");
        assert!(output.is_empty());
    }
}
