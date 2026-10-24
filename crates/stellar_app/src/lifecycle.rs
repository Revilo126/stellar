//! Contains all objects relating to the lifecycle of an [`crate::app::App`],
//! This includes the [`ScheduleLabel`]
//!

use std::{
    any::{TypeId, type_name},
    collections::HashMap,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicUsize, Ordering},
    },
};

pub trait ScheduleLabel: 'static + Send + Sync {
    fn name() -> &'static str {
        type_name::<Self>()
    }

    fn priority() -> isize {
        0
    }
}

static SCHEDULE_IDS: OnceLock<Mutex<HashMap<TypeId, ScheduleId>>> = OnceLock::new();
static NEXT_SCHEDULE_ID: AtomicUsize = AtomicUsize::new(0);

pub type ScheduleId = usize;

pub fn schedule_id<L: 'static>() -> ScheduleId {
    let ids = SCHEDULE_IDS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut ids = ids.lock().unwrap();

    let type_id = TypeId::of::<L>();

    if let Some(&id) = ids.get(&type_id) {
        return id;
    }

    let id = NEXT_SCHEDULE_ID.fetch_add(1, Ordering::Relaxed);

    ids.insert(type_id, id);

    id
}
