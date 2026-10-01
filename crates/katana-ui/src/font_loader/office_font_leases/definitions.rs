use super::super::office_faces::ResolvedFontFace;
use egui::{FontDefinitions, FontFamily};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct FaceIdentity {
    payload: usize,
    face_index: u32,
    family: String,
    weight: u16,
    bold: bool,
    italic: bool,
    monospaced: bool,
}

pub(super) struct SharedFace {
    pub(super) key: String,
    pub(super) family: FontFamily,
    pub(super) generic: FontFamily,
    pub(super) payload: Arc<egui::FontData>,
    pub(super) references: usize,
}

pub(super) struct LeaseFaces {
    pub(super) faces: Vec<ResolvedFontFace>,
    pub(super) identities: Vec<FaceIdentity>,
    pub(super) families: Vec<FontFamily>,
}

pub(super) struct ManagerState {
    pub(super) base: Arc<FontDefinitions>,
    pub(super) leases: BTreeMap<u64, LeaseFaces>,
    pub(super) shared: BTreeMap<FaceIdentity, SharedFace>,
    pub(super) next_id: u64,
    pub(super) epoch: u64,
}

pub(super) type SharedSnapshot = BTreeMap<FaceIdentity, String>;

pub(super) fn add_lease(
    state: &mut ManagerState,
    id: u64,
    faces: Vec<ResolvedFontFace>,
) -> LeaseFaces {
    let mut identities = Vec::with_capacity(faces.len());
    let mut families = Vec::with_capacity(faces.len());
    for (index, face) in faces.iter().enumerate() {
        let identity = identity(face);
        families.push(register_shared(state, id, index, face, &identity));
        identities.push(identity);
    }
    LeaseFaces {
        faces,
        identities,
        families,
    }
}

fn register_shared(
    state: &mut ManagerState,
    id: u64,
    index: usize,
    face: &ResolvedFontFace,
    identity: &FaceIdentity,
) -> FontFamily {
    if let Some(shared) = state.shared.get_mut(identity) {
        shared.references += 1;
        return shared.family.clone();
    }
    let key = unique_alias(state, id, index, face);
    let family = FontFamily::Name(key.clone().into());
    let generic = if face.monospaced {
        FontFamily::Monospace
    } else {
        FontFamily::Proportional
    };
    state.shared.insert(
        identity.clone(),
        SharedFace {
            key,
            family: family.clone(),
            generic,
            payload: face.payload.clone(),
            references: 1,
        },
    );
    family
}

fn unique_alias(state: &ManagerState, id: u64, index: usize, face: &ResolvedFontFace) -> String {
    let base = format!(
        "__document_face_{id}_{index}_{}_{}_{}_{}",
        face.family, face.weight, face.bold, face.italic
    );
    let mut suffix = 0_u64;
    loop {
        let key = if suffix == 0 {
            base.clone()
        } else {
            format!("{base}_{suffix}")
        };
        let family = FontFamily::Name(key.clone().into());
        let key_is_free = !state.base.font_data.contains_key(&key)
            && !state.shared.values().any(|shared| shared.key == key)
            && !state.base.families.contains_key(&family);
        if key_is_free {
            return key;
        }
        suffix += 1;
    }
}

pub(super) fn remove_references(state: &mut ManagerState, lease: &LeaseFaces) -> bool {
    let mut removed = false;
    for identity in &lease.identities {
        if let Some(shared) = state.shared.get_mut(identity) {
            shared.references -= 1;
            if shared.references == 0 {
                state.shared.remove(identity);
                removed = true;
            }
        }
    }
    removed
}

pub(super) fn identity(face: &ResolvedFontFace) -> FaceIdentity {
    FaceIdentity {
        payload: Arc::as_ptr(&face.payload) as usize,
        face_index: face.face_index,
        family: face.family.to_lowercase(),
        weight: face.weight,
        bold: face.bold,
        italic: face.italic,
        monospaced: face.monospaced,
    }
}

pub(super) fn same_base(left: &FontDefinitions, right: &FontDefinitions) -> bool {
    left.families == right.families
        && left.font_data.len() == right.font_data.len()
        && left.font_data.iter().all(|(key, data)| {
            right
                .font_data
                .get(key)
                .is_some_and(|other| Arc::ptr_eq(data, other))
        })
}

pub(super) fn same_faces(left: &[ResolvedFontFace], right: &[ResolvedFontFace]) -> bool {
    left.len() == right.len() && left.iter().zip(right).all(|(a, b)| same_face(a, b))
}

fn same_face(left: &ResolvedFontFace, right: &ResolvedFontFace) -> bool {
    left.request
        .family
        .eq_ignore_ascii_case(&right.request.family)
        && left.request.bold == right.request.bold
        && left.request.italic == right.request.italic
        && left.family.eq_ignore_ascii_case(&right.family)
        && left.weight == right.weight
        && left.bold == right.bold
        && left.italic == right.italic
        && left.monospaced == right.monospaced
        && left.face_index == right.face_index
        && Arc::ptr_eq(&left.payload, &right.payload)
}

pub(super) fn compose(state: &ManagerState) -> FontDefinitions {
    let mut definitions = (*state.base).clone();
    for shared in state.shared.values() {
        definitions
            .font_data
            .insert(shared.key.clone(), shared.payload.clone());
        let mut chain = vec![shared.key.clone()];
        if let Some(fallback) = definitions.families.get(&shared.generic) {
            chain.extend(fallback.iter().filter(|key| *key != &shared.key).cloned());
        }
        definitions.families.insert(shared.family.clone(), chain);
    }
    definitions
}

pub(super) fn shared_snapshot(state: &ManagerState) -> SharedSnapshot {
    state
        .shared
        .iter()
        .map(|(identity, face)| (identity.clone(), face.key.clone()))
        .collect()
}

pub(super) fn same_shared(state: &ManagerState, snapshot: &SharedSnapshot) -> bool {
    state.shared.len() == snapshot.len()
        && state
            .shared
            .iter()
            .all(|(identity, face)| snapshot.get(identity) == Some(&face.key))
}
