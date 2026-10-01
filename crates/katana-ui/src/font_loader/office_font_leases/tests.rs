use super::*;
use std::sync::Arc;
use support::*;

mod edge_cases;
mod support;

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
