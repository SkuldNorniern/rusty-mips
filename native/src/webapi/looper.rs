use super::GLOBAL_STATE;
use lazy_static::lazy_static;
use parking_lot::{Mutex, MutexGuard};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{spawn, JoinHandle};
use std::time::{Duration, Instant};

lazy_static! {
    static ref FLAG_RUN: AtomicBool = AtomicBool::new(false);
    static ref LOOPER: Mutex<Option<JoinHandle<Option<()>>>> = Mutex::new(None);
}

pub fn is_running() -> bool {
    FLAG_RUN.load(Ordering::Acquire)
}

pub fn start(allow_jit: bool) {
    FLAG_RUN.store(false, Ordering::Release);
    let mut guard = LOOPER.lock();
    // FIXME: Second unwrap (first is ok because it means thread panicked)
    if let Some(x) = guard.take() {
        x.join().unwrap().unwrap()
    }

    FLAG_RUN.store(true, Ordering::Release);
    *guard = Some(spawn(move || run_thread(allow_jit)));
}

pub fn stop() {
    FLAG_RUN.store(false, Ordering::Release);
    let mut guard = LOOPER.lock();
    // FIXME: Second unwrap (first is ok because it means thread panicked)
    if let Some(x) = guard.take() {
        x.join().unwrap().unwrap()
    }
}

fn run_thread(allow_jit: bool) -> Option<()> {
    let mut guard = GLOBAL_STATE.lock();
    let mut last_notify_time = Instant::now();

    // Use relaxed here. We acquire below there.
    while FLAG_RUN.load(Ordering::Relaxed) {
        //TODO: Handle exceptions (like overflow)

        if allow_jit {
            guard.as_mut()?.exec_silent().ok()?;
        } else {
            guard.as_mut()?.step_silent().ok()?;
        }

        let now = Instant::now();
        if 50 < (now - last_notify_time).as_millis() {
            guard.as_mut()?.notify_all();
            MutexGuard::unlock_fair(guard);

            // Code here runs mutex unlocked
            last_notify_time = now;

            // We need to check if we should exit before locking mutex
            // or we will get deadlock
            loop {
                if !FLAG_RUN.load(Ordering::Acquire) {
                    return Some(());
                }

                if let Some(x) = GLOBAL_STATE.try_lock_for(Duration::from_millis(10)) {
                    guard = x;
                    break;
                }
            }
        }
    }

    Some(())
}
