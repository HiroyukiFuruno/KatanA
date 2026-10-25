use super::*;
use egui::FontData;
use std::sync::Arc;
use support::*;

mod edge_cases;
mod support;

fn owned_ubuntu_payload(base: &egui::FontDefinitions) -> Arc<FontData> {
    Arc::new(FontData::from_owned(
        base.font_data["Ubuntu-Light"].font.to_vec(),
    ))
}

#[test]
fn independently_owned_equal_payloads_share_alias_and_epoch() {
    let (context, manager, base) = installed_context();
    let first_payload = owned_ubuntu_payload(&base);
    let second_payload = owned_ubuntu_payload(&base);
    assert!(!Arc::ptr_eq(&first_payload, &second_payload));
    assert_eq!(first_payload.font, second_payload.font);

    let mut first = manager.lease(&context);
    let mut second = manager.lease(&context);
    first.replace_faces(&[face(&first_payload, false)]);
    let first_epoch = epoch(&manager);
    second.replace_faces(&[face(&second_payload, false)]);

    assert_eq!(epoch(&manager), first_epoch);
    assert_eq!(
        first.family_for("Office Sans", false, false),
        second.family_for("Office Sans", false, false)
    );
}

#[test]
fn shared_payload_alias_survives_until_last_lease_drops() {
    let (context, manager, base) = installed_context();
    let payload = base.font_data["Ubuntu-Light"].clone();
    let mut first = manager.lease(&context);
    let mut second = manager.lease(&context);
    let regular = face(&payload, false);
    let bold = face(&payload, true);
    first.replace_faces(std::slice::from_ref(&regular));
    let first_epoch = epoch(&manager);
    second.replace_faces(std::slice::from_ref(&regular));
    assert_eq!(epoch(&manager), first_epoch);
    second.replace_faces(&[regular, bold]);
    assert_eq!(epoch(&manager), first_epoch + 1);

    let (shared_regular, bold_family) = assert_aliases(&context, &first, &second, &payload);
    first.close();
    assert_active_aliases(&context, &base, [&shared_regular, &bold_family], &payload);
    assert_layout(&context, shared_regular);
    drop(second);
    assert_same_definitions(&context, &base);
}

#[test]
fn unicode_family_lookup_preserves_style_matching() {
    let (context, manager, base) = installed_context();
    let payload = base.font_data["Ubuntu-Light"].clone();
    let mut lease = manager.lease(&context);
    lease.replace_faces(&[
        face_with(&payload, "école", false, false, false),
        face_with(&payload, "école", true, false, false),
    ]);

    let regular = lease
        .family_for("École", false, false)
        .expect("regular Unicode family alias");
    let bold = lease
        .family_for("École", true, false)
        .expect("bold Unicode family alias");
    assert_eq!(
        lease.family_for("école", false, false),
        Some(regular.clone())
    );
    assert_eq!(lease.family_for("école", true, false), Some(bold.clone()));
    assert_eq!(
        lease.family_for("E\u{301}cole", false, false),
        Some(regular.clone())
    );
    assert_ne!(regular, bold);
    assert!(lease.family_for("École", false, true).is_none());
}

#[test]
fn unicode_default_case_fold_preserves_sharp_s_identity() {
    let (context, manager, base) = installed_context();
    let payload = base.font_data["Ubuntu-Light"].clone();
    let mut lease = manager.lease(&context);
    lease.replace_faces(&[face_with(&payload, "Straße", false, false, false)]);

    assert!(lease.family_for("STRASSE", false, false).is_some());
}

#[test]
fn unicode_family_casing_change_preserves_epoch() {
    let (context, manager, base) = installed_context();
    let payload = base.font_data["Ubuntu-Light"].clone();
    let mut lease = manager.lease(&context);
    lease.replace_faces(&[face_with(&payload, "école", false, false, false)]);
    let initial_epoch = epoch(&manager);

    lease.replace_faces(&[face_with(&payload, "École", false, false, false)]);

    assert_eq!(epoch(&manager), initial_epoch);
}

#[test]
fn existing_registry_and_base_changes_preserve_active_faces() {
    let (context, manager, base) = installed_context();
    let payload = base.font_data["Ubuntu-Light"].clone();
    let mut lease = manager.lease(&context);
    let regular = face(&payload, false);
    lease.replace_faces(std::slice::from_ref(&regular));
    let initial_epoch = epoch(&manager);

    let reopened = DocumentFontLeaseManager::from_context(&context).expect("installed registry");
    assert_eq!(epoch(&reopened), initial_epoch);
    lease.replace_faces(std::slice::from_ref(&regular));
    assert_eq!(epoch(&manager), initial_epoch);

    let updated_base = Arc::new(base_with_marker(&base));
    manager.replace_base(&context, updated_base.clone());
    assert_eq!(epoch(&manager), initial_epoch + 1);
    assert!(
        context_definitions(&context)
            .font_data
            .contains_key(&alias_key(&lease, false))
    );
    drop(lease);
    assert_same_definitions(&context, &updated_base);
}
