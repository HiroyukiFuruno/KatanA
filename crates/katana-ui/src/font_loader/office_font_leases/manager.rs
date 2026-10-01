use super::super::office_faces::ResolvedFontFace;
use super::definitions::{
    LeaseFaces, ManagerState, add_lease, compose, remove_references, same_base, same_faces,
    same_shared, shared_snapshot,
};
use egui::{Context, FontDefinitions, FontFamily};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, MutexGuard},
};

const CONTEXT_DATA_ID: &str = "katana-office-document-font-leases";

#[derive(Clone)]
pub(crate) struct DocumentFontLeaseManager {
    pub(super) state: Arc<Mutex<ManagerState>>,
}

pub(crate) struct DocumentFontLease {
    context: Context,
    manager: DocumentFontLeaseManager,
    id: u64,
    active: bool,
}

impl DocumentFontLeaseManager {
    pub(crate) fn install_base(ctx: &Context, base: Arc<FontDefinitions>) -> Self {
        if let Some(manager) = Self::from_context(ctx) {
            manager.replace_base(ctx, base);
            return manager;
        }
        let manager = Self::new(base.clone());
        let id = egui::Id::new(CONTEXT_DATA_ID);
        ctx.data_mut(|data| data.insert_temp(id, manager.clone()));
        apply_snapshot(ctx, Some((*base).clone()));
        manager
    }

    pub(crate) fn from_context(ctx: &Context) -> Option<Self> {
        ctx.data_mut(|data| data.get_temp(egui::Id::new(CONTEXT_DATA_ID)))
    }

    fn new(base: Arc<FontDefinitions>) -> Self {
        Self {
            state: Arc::new(Mutex::new(ManagerState {
                base,
                leases: BTreeMap::new(),
                shared: BTreeMap::new(),
                next_id: 0,
                epoch: 0,
            })),
        }
    }

    pub(crate) fn replace_base(&self, ctx: &Context, base: Arc<FontDefinitions>) {
        let snapshot = {
            let mut state = self.lock_state();
            if same_base(&state.base, &base) {
                return;
            }
            state.base = base;
            state.epoch += 1;
            compose(&state)
        };
        apply_snapshot(ctx, Some(snapshot));
    }

    pub(crate) fn lease(&self, ctx: &Context) -> DocumentFontLease {
        let id = {
            let mut state = self.lock_state();
            let id = state.next_id;
            state.next_id += 1;
            state.leases.insert(
                id,
                LeaseFaces {
                    faces: Vec::new(),
                    identities: Vec::new(),
                    families: Vec::new(),
                },
            );
            id
        };
        DocumentFontLease {
            context: ctx.clone(),
            manager: self.clone(),
            id,
            active: true,
        }
    }

    fn lock_state(&self) -> MutexGuard<'_, ManagerState> {
        self.state
            .lock()
            .expect("document font lease state poisoned")
    }

    fn replace_faces(&self, id: u64, faces: &[ResolvedFontFace]) -> Option<FontDefinitions> {
        let mut state = self.lock_state();
        let current = state.leases.get(&id)?;
        if same_faces(&current.faces, faces) {
            return None;
        }
        let before = shared_snapshot(&state);
        let old = state.leases.remove(&id).expect("lease checked above");
        remove_references(&mut state, &old);
        let replacement = add_lease(&mut state, id, faces.to_vec());
        state.leases.insert(id, replacement);
        if same_shared(&state, &before) {
            return None;
        }
        state.epoch += 1;
        Some(compose(&state))
    }

    fn release(&self, id: u64) -> Option<FontDefinitions> {
        let mut state = self.lock_state();
        let before = shared_snapshot(&state);
        let lease = state.leases.remove(&id)?;
        remove_references(&mut state, &lease);
        if same_shared(&state, &before) {
            return None;
        }
        state.epoch += 1;
        Some(compose(&state))
    }
}

impl DocumentFontLease {
    pub(crate) fn replace_faces(&mut self, faces: &[ResolvedFontFace]) {
        apply_snapshot(&self.context, self.manager.replace_faces(self.id, faces));
    }

    pub(crate) fn family_for(&self, family: &str, bold: bool, italic: bool) -> Option<FontFamily> {
        let state = self.manager.lock_state();
        let lease = state.leases.get(&self.id)?;
        lease
            .faces
            .iter()
            .zip(&lease.families)
            .find(|(face, _)| {
                face.request.family.eq_ignore_ascii_case(family)
                    && face.request.bold == bold
                    && face.request.italic == italic
            })
            .map(|(_, family)| family.clone())
    }

    pub(crate) fn close(&mut self) {
        if !self.active {
            return;
        }
        self.active = false;
        apply_snapshot(&self.context, self.manager.release(self.id));
    }
}

impl Drop for DocumentFontLease {
    fn drop(&mut self) {
        self.close();
    }
}

fn apply_snapshot(ctx: &Context, snapshot: Option<FontDefinitions>) {
    if let Some(snapshot) = snapshot {
        ctx.set_fonts(snapshot);
        ctx.request_repaint();
    }
}
