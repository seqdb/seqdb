use std::{process::exit, sync::Arc};

use log::info;
use parking_lot::{RwLock, RwLockReadGuard};

/// Graceful shutdown coordinator for ensuring data consistency during program exit.
///
/// Uses a read-write lock to coordinate between operations and shutdown signals (e.g., Ctrl-C).
/// Operations hold read locks during critical sections, preventing shutdown until they complete.
#[derive(Default, Clone)]
pub struct Exit(Arc<RwLock<()>>);

impl Exit {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(())))
    }

    ///
    /// Only one handler throughout the program (and among all crates) can be set at once
    ///
    /// Make sure that no other crate sets one
    ///
    pub fn set_ctrlc_handler(&self) {
        let copy = self.0.clone();

        ctrlc::set_handler(move || {
            if copy.is_locked() {
                info!("Waiting to exit safely...");
            }
            let _lock = copy.write();
            info!("Exiting...");
            exit(0);
        })
        .expect("Error setting Ctrl-C handler");
    }

    pub fn lock(&self) -> RwLockReadGuard<'_, ()> {
        self.0.read()
    }
}
