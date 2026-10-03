use super::*;
use egui::{FontDefinitions, FontFamily, FontTweak};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[test]
fn duplicate_owned_payloads_release_weak_entries_after_external_faces_drop() {
    let (context, manager, base) = installed_context();
    let mut first = manager.lease(&context);
    let mut second = manager.lease(&context);

    let (canonical_weak, duplicate_weak) = {
        let first_payload = super::owned_ubuntu_payload(&base);
        let second_payload = super::owned_ubuntu_payload(&base);
        let canonical_weak = Arc::downgrade(&first_payload);
        let duplicate_weak = Arc::downgrade(&second_payload);
        first.replace_faces(&[face(&first_payload, false)]);
        second.replace_faces(&[face(&second_payload, false)]);
        (canonical_weak, duplicate_weak)
    };
    assert!(duplicate_weak.upgrade().is_none());
    assert!(canonical_weak.upgrade().is_some());

    let first_alias = first
        .family_for("Office Sans", false, false)
        .expect("first alias");
    let second_alias = second
        .family_for("Office Sans", false, false)
        .expect("second alias");
    assert_eq!(first_alias, second_alias);

    let alias = alias_key(&first, false);
    let definitions = context_definitions(&context);
    let context_payload = &definitions.font_data[&alias];
    let state = manager.state.lock().expect("lease state");
    assert_eq!(state.shared.len(), 1);
    let shared = state.shared.values().next().expect("shared face");
    assert_eq!(shared.references, 2);
    assert!(Arc::ptr_eq(context_payload, &shared.payload));
    assert!(state.leases.values().all(|lease| {
        lease
            .faces
            .iter()
            .all(|face| Arc::ptr_eq(&face.payload, &shared.payload))
    }));
    drop(state);
    drop(definitions);

    first.close();
    assert_eq!(manager.state.lock().expect("lease state").shared.len(), 1);
    second.close();
    assert!(manager.state.lock().expect("lease state").shared.is_empty());
    assert_same_definitions(&context, &base);
    assert!(canonical_weak.upgrade().is_none());
}

#[test]
fn different_payload_index_style_and_tweak_do_not_share() {
    let (context, manager, base) = installed_context();
    let bytes = base.font_data["Ubuntu-Light"].font.to_vec();

    let changed_payload = Arc::new(egui::FontData::from_owned(
        base.font_data["Hack"].font.to_vec(),
    ));

    let mut indexed = egui::FontData::from_owned(bytes.clone());
    indexed.index = 1;
    let indexed_payload = Arc::new(indexed);

    let tweaked_payload = Arc::new(egui::FontData::from_owned(bytes).tweak(FontTweak {
        scale: 1.1,
        ..Default::default()
    }));
    let canonical_payload = super::owned_ubuntu_payload(&base);
    let regular = face(&canonical_payload, false);
    assert_ne!(
        super::super::definitions::identity(&regular),
        super::super::definitions::identity(&face(&indexed_payload, false))
    );
    let mut indexed_face = regular.clone();
    indexed_face.face_index = 1;
    assert_ne!(
        super::super::definitions::identity(&regular),
        super::super::definitions::identity(&indexed_face)
    );

    let mut lease = manager.lease(&context);
    lease.replace_faces(&[
        face(&canonical_payload, false),
        face(&changed_payload, false),
        face(&canonical_payload, true),
        face(&tweaked_payload, false),
    ]);

    let state = manager.state.lock().expect("lease state");
    assert_eq!(state.shared.len(), 4);
    assert!(state.shared.values().all(|shared| shared.references == 1));
    drop(state);

    assert_ne!(alias_key(&lease, false), alias_key(&lease, true));
    drop(lease);
    assert!(manager.state.lock().expect("lease state").shared.is_empty());
    assert_same_definitions(&context, &base);
}

#[test]
fn missing_family_and_style_are_not_resolved() {
    let (context, manager, base) = installed_context();
    let payload = base.font_data["Ubuntu-Light"].clone();
    let mut lease = manager.lease(&context);
    lease.replace_faces(&[face(&payload, false)]);

    assert!(lease.family_for("Other Sans", false, false).is_none());
    assert!(lease.family_for("Office Sans", true, false).is_none());
    assert!(lease.family_for("Office Sans", false, true).is_none());
    drop(lease);
    assert_same_definitions(&context, &base);
}

#[test]
fn close_twice_and_drop_restore_context_fonts() {
    let (context, manager, base) = installed_context();
    let payload = base.font_data["Ubuntu-Light"].clone();
    let mut lease = manager.lease(&context);
    lease.replace_faces(&[face(&payload, false)]);
    assert_ne!(
        context_definitions(&context).font_data.len(),
        base.font_data.len()
    );

    lease.close();
    lease.close();
    assert_same_definitions(&context, &base);
    drop(lease);
    assert_same_definitions(&context, &base);
}

#[test]
fn active_panes_survive_base_update_until_last_drop() {
    let (context, manager, base) = installed_context();
    let payload = base.font_data["Ubuntu-Light"].clone();
    let mut first = manager.lease(&context);
    let mut second = manager.lease(&context);
    first.replace_faces(&[face(&payload, false)]);
    second.replace_faces(&[face(&payload, false)]);
    let alias = first
        .family_for("Office Sans", false, false)
        .expect("alias");
    let updated = Arc::new(base_with_marker(&base));

    manager.replace_base(&context, updated.clone());
    assert_payload(&context, &alias, &payload);
    first.close();
    assert_payload(&context, &alias, &payload);
    drop(second);
    assert_same_definitions(&context, &updated);
}

#[test]
fn monospace_alias_preserves_the_updated_generic_chain() {
    let (context, manager, base) = installed_context();
    let payload = base.font_data["Ubuntu-Light"].clone();
    let mut lease = manager.lease(&context);
    lease.replace_faces(&[face_with(&payload, "Office Mono", false, false, true)]);
    let alias = lease
        .family_for("Office Mono", false, false)
        .expect("monospace alias");
    let definitions = context_definitions(&context);
    let chain = &definitions.families[&alias];

    assert_payload(&context, &alias, &payload);
    assert_eq!(&chain[1..], base.families[&FontFamily::Monospace]);
    assert_eq!(
        definitions.families[&FontFamily::Monospace],
        base.families[&FontFamily::Monospace]
    );
}

#[test]
fn alias_collision_keeps_existing_font_and_chooses_unique_key() {
    let mut base = FontDefinitions::default();
    let collision = "__document_face_0_0_Office Sans_400_false_false";
    let payload = base.font_data["Ubuntu-Light"].clone();
    base.font_data.insert(collision.into(), payload.clone());
    let base = Arc::new(base);
    let context = egui::Context::default();
    let manager = DocumentFontLeaseManager::install_base(&context, base);
    let mut lease = manager.lease(&context);
    lease.replace_faces(&[face(&payload, false)]);
    let alias = lease
        .family_for("Office Sans", false, false)
        .expect("alias");
    let FontFamily::Name(alias_key) = alias else {
        panic!("lease alias must be named");
    };

    assert_ne!(alias_key.as_ref(), collision);
    assert!(Arc::ptr_eq(
        &context_definitions(&context).font_data[collision],
        &payload
    ));
}

#[test]
fn ten_style_replacements_update_context_and_restore_base() {
    let (context, manager, base) = installed_context();
    let payload = base.font_data["Ubuntu-Light"].clone();
    let mut lease = manager.lease(&context);
    lease.replace_faces(&[face(&payload, false)]);
    let initial_epoch = epoch(&manager);

    for index in 0..10 {
        let bold = index % 2 == 0;
        lease.replace_faces(&[face(&payload, bold)]);
        assert!(lease.family_for("Office Sans", bold, false).is_some());
        assert!(lease.family_for("Office Sans", !bold, false).is_none());
    }
    assert_eq!(epoch(&manager), initial_epoch + 10);
    drop(lease);
    assert_same_definitions(&context, &base);
}

#[test]
fn changed_font_snapshots_request_repaint_but_noops_do_not() {
    let context = egui::Context::default();
    let repaint_count = Arc::new(AtomicUsize::new(0));
    let callback_count = repaint_count.clone();
    context.set_request_repaint_callback(move |_| {
        callback_count.fetch_add(1, Ordering::SeqCst);
    });
    let base = Arc::new(FontDefinitions::default());
    let manager = DocumentFontLeaseManager::install_base(&context, base.clone());
    assert!(repaint_count.swap(0, Ordering::SeqCst) > 0);
    settle_repaints(&context);
    repaint_count.store(0, Ordering::SeqCst);

    let payload = base.font_data["Ubuntu-Light"].clone();
    let mut lease = manager.lease(&context);
    let face = face(&payload, false);
    lease.replace_faces(std::slice::from_ref(&face));
    assert!(repaint_count.swap(0, Ordering::SeqCst) > 0);
    settle_repaints(&context);
    repaint_count.store(0, Ordering::SeqCst);

    lease.replace_faces(std::slice::from_ref(&face));
    manager.replace_base(&context, base);
    assert_eq!(repaint_count.load(Ordering::SeqCst), 0);

    settle_repaints(&context);
    repaint_count.store(0, Ordering::SeqCst);
    lease.close();
    assert!(repaint_count.load(Ordering::SeqCst) > 0);
}

#[test]
fn changed_base_snapshot_requests_repaint() {
    let context = egui::Context::default();
    let repaint_count = Arc::new(AtomicUsize::new(0));
    let callback_count = repaint_count.clone();
    context.set_request_repaint_callback(move |_| {
        callback_count.fetch_add(1, Ordering::SeqCst);
    });
    let base = Arc::new(FontDefinitions::default());
    let manager = DocumentFontLeaseManager::install_base(&context, base.clone());
    settle_repaints(&context);
    repaint_count.store(0, Ordering::SeqCst);

    manager.replace_base(&context, Arc::new(base_with_marker(&base)));
    assert!(repaint_count.load(Ordering::SeqCst) > 0);
}

fn settle_repaints(context: &egui::Context) {
    for _ in 0..8 {
        if !context.has_requested_repaint() {
            return;
        }
        let mut output = context.run_ui(Default::default(), |_| {});
        output.textures_delta.clear();
    }
    assert!(
        !context.has_requested_repaint(),
        "egui repaint request did not settle across frames"
    );
}
