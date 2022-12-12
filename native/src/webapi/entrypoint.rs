use super::state::State;
use super::util::take_state;
use crate::component::RegisterName;
use neon::prelude::*;

const API_VERSION: u32 = 1;

fn log_console<'c, C: Context<'c>>(cx: &mut C, msg: impl AsRef<str>) {
    let msg = msg.as_ref();
    let _ = cx
        .global()
        .get(cx, "console")
        .and_then(|x: Handle<JsObject>| x.get(cx, "log"))
        .and_then(|x: Handle<JsFunction>| x.call_with(cx).arg(cx.string(msg)).apply(cx))
        .map(|_: Handle<JsUndefined>| ());
}

fn init(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let callback = cx.argument::<JsFunction>(0)?.root(&mut cx);

    let mut guard = super::GLOBAL_STATE.lock();
    if guard.is_some() {
        // Re-init is not an error because refreshing the page causes them.
        log_console(&mut cx, "Re-initializing native module!");
    }
    *guard = Some(State::new(cx.channel(), callback));
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

fn reset(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    take_state(&mut cx)?.reset();
    Ok(JsUndefined::new(&mut cx))
}

fn assemble(mut cx: FunctionContext) -> JsResult<JsValue> {
    let code = cx.argument::<JsString>(0)?.value(&mut cx);
    match take_state(&mut cx)?.assemble(&code) {
        Ok(_) => Ok(cx.null().upcast()),
        Err(e) => Ok(cx.string(e).upcast()),
    }
}

fn edit_register(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let idx = cx.argument::<JsNumber>(0)?.value(&mut cx) as u8;
    let val = cx.argument::<JsNumber>(1)?.value(&mut cx) as u32;

    if let Some(x) = RegisterName::try_from_num(idx) {
        let mut state = take_state(&mut cx)?;
        let _ = state.edit_register(x, val);
    }

    Ok(cx.undefined())
}

pub fn register_functions(cx: &mut ModuleContext) -> NeonResult<()> {
    cx.export_function("init", init)?;
    cx.export_function("finalize", finalize)?;
    cx.export_function("reset", reset)?;
    cx.export_function("assemble", assemble)?;
    cx.export_function("editRegister", edit_register)?;
    Ok(())
}
