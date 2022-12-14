use super::state::State;
use super::util::take_state;
use crate::component::RegisterName;
use crate::memory::EndianMode;
use crate::webapi::util::log_console;
use neon::prelude::*;
use neon::types::buffer::TypedArray;
use std::cell::RefCell;

const API_VERSION: u32 = 1;

thread_local!(static READ_MEM_BUFFER: RefCell<Vec<u8>> = RefCell::new(vec![0; 4096]));

fn init(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let callback = cx.argument::<JsFunction>(0)?.root(&mut cx);

    let mut guard = super::GLOBAL_STATE.lock();
    if let Some(x) = guard.take() {
        // Re-init is not an error because refreshing the page causes them.
        log_console(&mut cx, "Re-initializing native module!");
        let _ = x.stop();
    }
    *guard = Some(State::new(cx.channel(), callback));
    drop(guard);

    Ok(JsNumber::new(&mut cx, API_VERSION))
}

fn finalize(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut guard = super::GLOBAL_STATE.lock();
    if let Some(x) = guard.take() {
        let _ = x.stop();
        drop(x);
    }
    drop(guard);

    Ok(JsUndefined::new(&mut cx))
}

fn reset(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut state = take_state(&mut cx)?;

    let mut updates = state.stop();
    updates |= state.reset();
    state.notify(updates);

    Ok(JsUndefined::new(&mut cx))
}

fn assemble(mut cx: FunctionContext) -> JsResult<JsValue> {
    let code = cx.argument::<JsString>(0)?.value(&mut cx);
    let endian = cx.argument::<JsString>(1)?.value(&mut cx);

    let endian = match endian.as_str() {
        "big" => EndianMode::Big,
        "little" => EndianMode::Little,
        _ => EndianMode::native(),
    };

    let mut state = take_state(&mut cx)?;

    match state.assemble(&code, endian) {
        Ok(x) => {
            state.notify(x);
            Ok(cx.null().upcast())
        }
        Err(e) => Ok(cx.string(e).upcast()),
    }
}

fn edit_register(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let idx = cx.argument::<JsNumber>(0)?.value(&mut cx) as u8;
    let val = cx.argument::<JsNumber>(1)?.value(&mut cx) as u32;

    if let Some(x) = RegisterName::try_from_num(idx) {
        let mut state = take_state(&mut cx)?;
        let updates = state.edit_register(x, val);
        state.notify(updates);
    }

    Ok(cx.undefined())
}

fn read_memory(mut cx: FunctionContext) -> JsResult<JsValue> {
    let page_idx = cx.argument::<JsNumber>(0)?.value(&mut cx) as i32;
    let mut dst = cx.argument::<JsUint8Array>(1)?;
    if !(0..1048576).contains(&page_idx) {
        return Ok(cx.null().upcast());
    }

    let mut altered = false;
    let state = take_state(&mut cx)?;
    let output = dst.as_mut_slice(&mut cx);
    if output.len() != 4096 {
        panic!("buffer length must be 4096 bytes");
    }

    READ_MEM_BUFFER.with(|x| {
        let mut buffer = x.borrow_mut();
        state.read_memory(page_idx as u32, &mut buffer);
        if output != buffer.as_slice() {
            altered = true;
            output.copy_from_slice(&buffer);
        }
    });

    Ok(if altered {
        dst.upcast()
    } else {
        cx.null().upcast()
    })
}

fn step(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut state = take_state(&mut cx)?;

    match state.step() {
        Ok(x) => state.notify(x),
        Err(e) => log_console(&mut cx, e),
    }

    Ok(cx.undefined())
}

fn run(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let allow_jit = cx.argument::<JsBoolean>(0)?.value(&mut cx);

    let state = take_state(&mut cx)?;
    let updates = state.run(allow_jit);
    state.notify(updates);

    Ok(cx.undefined())
}

fn stop(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let state = take_state(&mut cx)?;

    let updates = state.stop();
    state.notify(updates);

    Ok(cx.undefined())
}

fn get_native_endian(mut cx: FunctionContext) -> JsResult<JsString> {
    let endian = match EndianMode::native() {
        EndianMode::Little => "little",
        EndianMode::Big => "big",
    };

    Ok(cx.string(endian))
}

fn convert_to_pipeline(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut state = take_state(&mut cx)?;
    let updates = state.convert_to_pipeline();
    state.notify(updates);
    Ok(cx.undefined())
}

pub fn register_functions(cx: &mut ModuleContext) -> NeonResult<()> {
    cx.export_function("init", init)?;
    cx.export_function("finalize", finalize)?;
    cx.export_function("reset", reset)?;
    cx.export_function("assemble", assemble)?;
    cx.export_function("editRegister", edit_register)?;
    cx.export_function("readMemory", read_memory)?;
    cx.export_function("step", step)?;
    cx.export_function("run", run)?;
    cx.export_function("stop", stop)?;
    cx.export_function("getNativeEndian", get_native_endian)?;
    cx.export_function("convertToPipeline", convert_to_pipeline)?;
    Ok(())
}
