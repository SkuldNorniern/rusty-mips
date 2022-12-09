use lazy_static::lazy_static;

mod entrypoint;
mod state;
mod util;

pub use entrypoint::register_functions;

lazy_static! {
    static ref GLOBAL_STATE: parking_lot::Mutex<Option<state::State>> =
        parking_lot::Mutex::new(None);
}
