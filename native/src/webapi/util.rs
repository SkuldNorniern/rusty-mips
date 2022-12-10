use super::state::State;
use neon::prelude::*;
use parking_lot::{MappedMutexGuard, MutexGuard};

pub fn take_state(cx: &mut FunctionContext) -> NeonResult<MappedMutexGuard<'static, State>> {
    let guard = super::GLOBAL_STATE.lock();
    if guard.is_none() {
        return Err(cx.throw_error("simulator not initialized")?);
    }

    Ok(MutexGuard::map(guard, |x| x.as_mut().unwrap()))
}
