use super::state::State;
use neon::prelude::*;

const API_VERSION: u32 = 1;

fn init(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let mut guard = super::GLOBAL_STATE.lock();
    if guard.is_some() {
        // Re-init is not an error because refreshing the page causes them.
        let msg = "Re-initializing native module!";
        let _ = cx
            .global()
            .get(&mut cx, "console")
            .and_then(|x: Handle<JsObject>| x.get(&mut cx, "log"))
            .and_then(|x: Handle<JsFunction>| x.call_with(&cx).arg(cx.string(msg)).apply(&mut cx))
            .map(|_: Handle<JsUndefined>| ());
    }
    *guard = Some(State::new());
    drop(guard);

    Ok(JsNumber::new(&mut cx, API_VERSION))
}

fn finalize(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut guard = super::GLOBAL_STATE.lock();
    if guard.is_none() {
        return Err(cx.throw_error("simulator not initialized, but tried to finalize")?);
    }
    guard.take();
    drop(guard);

    Ok(JsUndefined::new(&mut cx))
}

pub fn register_functions(cx: &mut ModuleContext) -> NeonResult<()> {
    cx.export_function("init", init)?;
    cx.export_function("finalize", finalize)?;
    Ok(())
}
