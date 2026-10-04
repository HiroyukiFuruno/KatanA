mod face_payload;
mod font_file;
mod font_metadata;
mod resolver;
mod resolver_debug;
mod resolver_scan;
mod resolver_selection;
mod types;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod ttc_payload_regression_tests;

#[cfg(test)]
pub(super) mod ttc_payload_assertions;

#[cfg(test)]
pub(super) mod ttc_payload_fixture;

#[cfg(test)]
mod metadata_tests;

#[cfg(test)]
mod metadata_alias_tests;

#[cfg(test)]
mod sfnt_fixture;

#[cfg(test)]
mod cancellation_tests;

#[cfg(test)]
mod installed_font_tests;

pub(crate) use resolver::FontFaceResolver;
pub(crate) use types::{
    FontFaceRequest, FontFaceResolution, FontFaceResolutionDiagnostic, FontPayloadDigest,
    ResolvedFontFace,
};

#[cfg(test)]
pub(crate) fn installed_regular_bold_pair() -> (String, std::path::PathBuf, std::path::PathBuf) {
    installed_font_tests::installed_regular_bold_pair()
}

#[cfg(test)]
pub(crate) fn installed_italic_face() -> (String, std::path::PathBuf) {
    installed_font_tests::installed_italic_face()
}
