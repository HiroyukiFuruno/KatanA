use super::*;
use std::sync::mpsc;

const BACKGROUND: egui::Color32 = crate::theme_bridge::WHITE;

fn work(loader: &LocalImageLoader, path: &str) -> Work {
    Work {
        owner: Arc::downgrade(&loader.inner),
        result_tx: loader.inner.result_tx.clone(),
        key: RequestKey {
            path: PathBuf::from(path),
            background: BACKGROUND,
        },
        generation: loader.generation(),
        revision: 0,
    }
}

#[test]
fn full_queue_stays_pending_and_retries_after_slot_is_released() {
    let loader = LocalImageLoader::default();
    let (sender, receiver) = mpsc::sync_channel(1);
    sender
        .try_send(work(&loader, "issue357-occupier.png"))
        .expect("occupy bounded test queue");

    let key = RequestKey {
        path: PathBuf::from("issue357-retried.png"),
        background: BACKGROUND,
    };
    let actual_work = work(&loader, "issue357-retried.png");
    loader
        .inner
        .pending
        .lock()
        .expect("pending lock")
        .insert(key.clone());

    assert!(matches!(
        loader.enqueue_to(&sender, &key, actual_work),
        LocalImageStatus::Pending
    ));
    assert!(
        !loader
            .inner
            .pending
            .lock()
            .expect("pending lock")
            .contains(&key),
        "a full queue must not leave a phantom in-flight marker"
    );
    assert!(loader.cached_status(&key).is_none());

    receiver.try_recv().expect("release queue slot");
    loader
        .inner
        .pending
        .lock()
        .expect("pending lock")
        .insert(key.clone());
    assert!(matches!(
        loader.enqueue_to(&sender, &key, work(&loader, "issue357-retried.png")),
        LocalImageStatus::Pending
    ));
    let accepted = receiver.try_recv().expect("retried work accepted");
    assert_eq!(accepted.key.path, key.path);
    assert_eq!(accepted.key.background, key.background);
    assert!(
        loader
            .inner
            .pending
            .lock()
            .expect("pending lock")
            .contains(&key),
        "accepted work remains tracked until its result is polled"
    );
}

#[test]
fn disconnected_queue_is_reported_as_unavailable_not_capacity_pressure() {
    let loader = LocalImageLoader::default();
    let (sender, receiver) = mpsc::sync_channel::<Work>(1);
    drop(receiver);
    let key = RequestKey {
        path: PathBuf::from("issue357-disconnected.png"),
        background: BACKGROUND,
    };
    loader
        .inner
        .pending
        .lock()
        .expect("pending lock")
        .insert(key.clone());

    let LocalImageStatus::Failed(error) =
        loader.enqueue_to(&sender, &key, work(&loader, "issue357-disconnected.png"))
    else {
        panic!("a disconnected decoder pool is unavailable");
    };
    assert_eq!(error, "image loader unavailable");
    assert!(
        !loader
            .inner
            .pending
            .lock()
            .expect("pending lock")
            .contains(&key)
    );
}
