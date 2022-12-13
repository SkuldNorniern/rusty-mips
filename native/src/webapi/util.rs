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

pub fn log_console<'c, C: Context<'c>>(cx: &mut C, msg: impl AsRef<str>) {
    let msg = msg.as_ref();
    let _ = cx
        .global()
        .get(cx, "console")
        .and_then(|x: Handle<JsObject>| x.get(cx, "log"))
        .and_then(|x: Handle<JsFunction>| x.call_with(cx).arg(cx.string(msg)).apply(cx))
        .map(|_: Handle<JsUndefined>| ());
}
