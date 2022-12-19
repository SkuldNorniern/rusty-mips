mod entrypoint;
mod looper;
mod state;
mod updates;
mod util;

lazy_static::lazy_static! {
    static ref GLOBAL_STATE: parking_lot::Mutex<Option<state::State>> =
        parking_lot::Mutex::new(None);
}
