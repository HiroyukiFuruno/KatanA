use super::WorkerMemoryState;
use super::{DocumentWorkerLease, DocumentWorkerLifecycle};
use std::sync::atomic::{AtomicUsize, Ordering};

struct OwnedWork<'a>(&'a AtomicUsize);

impl Drop for OwnedWork<'_> {
    fn drop(&mut self) {
        assert_eq!(self.0.load(Ordering::Acquire), 1);
    }
}

#[test]
fn worker_lease_remains_live_until_owned_work_has_dropped() {
    fn run(work: OwnedWork<'_>) {
        drop(work);
    }

    let state = WorkerMemoryState::new();
    let counter = &state.counter;
    std::thread::scope(|scope| {
        let lease = DocumentWorkerLease::acquire(&state);
        assert_eq!(counter.load(Ordering::Acquire), 1);
        let work = OwnedWork(counter);
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let (finish_tx, finish_rx) = std::sync::mpsc::channel();
        let worker = scope.spawn(move || {
            let _worker_lifetime = lease;
            started_tx.send(()).expect("worker started");
            finish_rx.recv().expect("finish requested");
            run(work);
        });
        started_rx.recv().expect("worker started");
        assert_eq!(counter.load(Ordering::Acquire), 1);
        finish_tx.send(()).expect("finish requested");
        worker.join().expect("worker completed");
    });
    assert_eq!(counter.load(Ordering::Acquire), 0);
}

#[test]
fn worker_lease_releases_on_worker_panic() {
    fn run(_work: OwnedWork<'_>) {
        panic!("worker failure");
    }

    let state = WorkerMemoryState::new();
    let counter = &state.counter;
    std::thread::scope(|scope| {
        let lease = DocumentWorkerLease::acquire(&state);
        let work = OwnedWork(counter);
        let worker = scope.spawn(move || {
            let _worker_lifetime = lease;
            run(work);
        });
        assert!(worker.join().is_err());
    });
    assert_eq!(counter.load(Ordering::Acquire), 0);
}

#[test]
fn process_worker_lease_is_observable_without_a_pane() {
    let lease = DocumentWorkerLease::start();
    assert!(DocumentWorkerLifecycle::live_count() >= 1);
    drop(lease);
}

#[test]
fn worker_lease_releases_when_thread_creation_fails() {
    let state = WorkerMemoryState::new();
    let counter = &state.counter;
    std::thread::scope(|scope| {
        let lease = DocumentWorkerLease::acquire(&state);
        let worker_inputs = (OwnedWork(counter), lease);
        let start = std::panic::catch_unwind(|| {
            std::thread::Builder::new()
                .stack_size(usize::MAX)
                .spawn_scoped(scope, move || {
                    let (work, _worker_lifetime) = std::convert::identity(worker_inputs);
                    drop(work);
                })
        });
        assert!(!matches!(start, Ok(Ok(_))));
    });
    assert_eq!(counter.load(Ordering::Acquire), 0);
}

#[test]
fn lifecycle_spawn_observes_worker_while_owned_work_runs() {
    let (started_tx, started_rx) = std::sync::mpsc::channel();
    let worker = DocumentWorkerLifecycle::spawn("lifecycle-regression".to_owned(), move || {
        started_tx
            .send(DocumentWorkerLifecycle::live_count())
            .expect("worker started");
    })
    .expect("worker spawned");
    assert!(started_rx.recv().expect("worker started") >= 1);
    worker.join().expect("worker completed");
}

#[test]
fn worker_lease_releases_when_unstarted_closure_is_dropped() {
    let state = WorkerMemoryState::new();
    let counter = &state.counter;
    let lease = DocumentWorkerLease::acquire(&state);
    let worker_inputs = (OwnedWork(counter), lease);
    let worker = move || {
        let (work, _worker_lifetime) = std::convert::identity(worker_inputs);
        drop(work);
    };
    assert_eq!(counter.load(Ordering::Acquire), 1);
    drop(worker);
    assert_eq!(counter.load(Ordering::Acquire), 0);
}
