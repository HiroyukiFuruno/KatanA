use super::super::office_faces::{FontPayloadDigest, ResolvedFontFace};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct FaceIdentity {
    payload: FontPayloadDigest,
    payload_index: u32,
    tweak: String,
    face_index: u32,
    family: String,
    weight: u16,
    bold: bool,
    italic: bool,
    monospaced: bool,
}

pub(super) fn identity(face: &ResolvedFontFace) -> FaceIdentity {
    FaceIdentity {
        payload: face.payload_digest,
        payload_index: face.payload.index,
        tweak: format!("{:?}", face.payload.tweak),
        face_index: face.face_index,
        family: face.family.to_lowercase(),
        weight: face.weight,
        bold: face.bold,
        italic: face.italic,
        monospaced: face.monospaced,
    }
}
