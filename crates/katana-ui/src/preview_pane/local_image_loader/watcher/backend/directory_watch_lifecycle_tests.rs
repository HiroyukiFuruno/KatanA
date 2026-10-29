use super::DirectoryWatcher;
use super::tests::{EVENT_TIMEOUT, watcher};
use super::worker::{DirectoryCommand, DirectoryWatchLoop};
use notify::{Config, RecursiveMode, Watcher};
use std::sync::mpsc;

#[test]
fn unwatch_acknowledges_removal_before_later_filesystem_loss() {
    let root = tempfile::tempdir().expect("directory fixture");
    let path = root.path().join("watched");
    std::fs::create_dir(&path).expect("watched directory");
    let (mut watcher, events) = watcher();
    watcher
        .watch(&path, RecursiveMode::NonRecursive)
        .expect("directory registration");
    watcher
        .unwatch(&path)
        .expect("directory removal acknowledged");
    std::fs::remove_dir(&path).expect("remove unwatched directory");
    let error = watcher
        .unwatch(&path)
        .expect_err("unknown directory rejected");
    assert!(matches!(error.kind, notify::ErrorKind::WatchNotFound));
    drop(watcher);
    assert!(matches!(
        events.try_recv(),
        Err(mpsc::TryRecvError::Disconnected)
    ));
}

#[test]
fn drop_stops_the_owned_thread_and_closes_its_event_sender() {
    let root = tempfile::tempdir().expect("directory fixture");
    let (mut watcher, events) = watcher();
    watcher
        .watch(root.path(), RecursiveMode::NonRecursive)
        .expect("directory registration");
    watcher
        .watch(root.path(), RecursiveMode::NonRecursive)
        .expect("duplicate registration");
    drop(watcher);
    assert!(matches!(
        events.try_recv(),
        Err(mpsc::TryRecvError::Disconnected)
    ));
}

#[test]
fn command_channel_closure_terminates_the_owned_loop() {
    let (commands, receiver) = mpsc::channel();
    let (finished, completion) = mpsc::channel();
    let thread = std::thread::spawn(move || {
        DirectoryWatchLoop::new(|_| {}).run(receiver, EVENT_TIMEOUT);
        finished.send(()).expect("loop termination receipt");
    });
    drop(commands);
    completion
        .recv_timeout(EVENT_TIMEOUT)
        .expect("closed command channel stops loop");
    thread.join().expect("directory loop thread");
}

#[test]
fn closed_reply_channel_does_not_prevent_stop() {
    let root = tempfile::tempdir().expect("directory fixture");
    let path = root.path().to_path_buf();
    let (commands, receiver) = mpsc::channel();
    let thread = std::thread::spawn(move || {
        DirectoryWatchLoop::new(|_| {}).run(receiver, EVENT_TIMEOUT);
    });
    let (reply, result) = mpsc::channel();
    drop(result);
    commands
        .send(DirectoryCommand::Watch(path, reply))
        .expect("watch request");
    commands.send(DirectoryCommand::Stop).expect("stop request");
    thread
        .join()
        .expect("closed reply does not orphan directory thread");
}

#[test]
fn stopped_worker_rejects_subsequent_registration() {
    let root = tempfile::tempdir().expect("directory fixture");
    let (mut watcher, _) = watcher();
    watcher
        .commands
        .send(DirectoryCommand::Stop)
        .expect("stop request");
    watcher
        .thread
        .take()
        .expect("owned thread")
        .join()
        .expect("stopped thread");
    assert!(
        watcher
            .watch(root.path(), RecursiveMode::NonRecursive)
            .is_err()
    );
}

#[test]
fn unsupported_poll_configuration_is_rejected() {
    assert_eq!(DirectoryWatcher::kind(), notify::WatcherKind::PollWatcher);
    for config in [
        Config::default().with_manual_polling(),
        Config::default().with_poll_interval(std::time::Duration::ZERO),
        Config::default().with_compare_contents(true),
    ] {
        assert!(DirectoryWatcher::new(|_| {}, config).is_err());
    }
}

#[test]
fn callback_panic_is_reaped_without_panicking_from_drop() {
    let root = tempfile::tempdir().expect("directory fixture");
    let path = root.path().join("watched");
    std::fs::create_dir(&path).expect("watched directory");
    let (entered, receipt) = mpsc::channel();
    let mut watcher = DirectoryWatcher::new(
        move |_| {
            entered.send(()).expect("callback receipt");
            panic!("expected directory callback panic");
        },
        Config::default().with_poll_interval(super::super::WINDOWS_IMAGE_POLL_INTERVAL),
    )
    .expect("directory watcher");
    watcher
        .watch(&path, RecursiveMode::NonRecursive)
        .expect("directory registration");
    std::fs::remove_dir(&path).expect("directory loss");
    receipt
        .recv_timeout(EVENT_TIMEOUT)
        .expect("callback entered");
    drop(watcher);
}
