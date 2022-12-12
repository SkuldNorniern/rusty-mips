use crate::assembler::assemble;
use crate::component::RegisterName;
use crate::interpreter::Interpreter;
use crate::memory::{create_memory, create_empty_memory, EndianMode};
use neon::prelude::*;
use std::sync::Arc;

#[derive(Debug)]
pub struct State {
    channel: Channel,
    callback: Arc<Root<JsFunction>>,
    inner: Inner,
}

#[derive(Debug)]
struct Inner {
    interpreter: Interpreter,
}

impl Default for Inner {
    fn default() -> Self {
        Inner {
            interpreter: Interpreter::new(create_empty_memory(EndianMode::native())),
        }
    }
}

impl State {
    pub fn new(ch: Channel, callback: Root<JsFunction>) -> Self {
        let ret = State {
            channel: ch,
            callback: Arc::new(callback),
            inner: Default::default(),
        };
        ret.notify_all();
        ret
    }

    pub fn reset(&mut self) {
        self.inner = Default::default();
        self.notify_all();
    }

    pub fn assemble(&mut self, code: &str) -> Result<(), String> {
        let segs = assemble(EndianMode::native(), code).map_err(|e| e.to_string())?;
        let mem = create_memory(EndianMode::native(), &segs);
        self.inner.interpreter = Interpreter::new(mem);
        self.notify_all();
        Ok(())
    }

    pub fn edit_register(&mut self, r: RegisterName, val: u32) -> Result<(), ()> {
        self.inner.interpreter.set_reg(r, val);

        self.notify_all();

        Ok(())
    }

    fn notify_all(&self) {
        let callback = self.callback.clone();

        let regs = self.inner.capture_regs();

        self.channel.send(move |mut cx| {
            let regs = js_array_numbers(&mut cx, &regs)?;
            let obj = cx.empty_object();
            obj.set(&mut cx, "regs", regs)?;

            callback
                .to_inner(&mut cx)
                .call_with(&cx)
                .arg(obj)
                .exec(&mut cx)
        });
    }
}

impl Inner {
    fn capture_regs(&self) -> [u32; 32] {
        let mut ret = [0; 32];
        self.interpreter.read_all_reg(&mut ret);
        ret
    }
}

fn js_array_numbers<'a, C: Context<'a>>(cx: &mut C, arr: &[u32]) -> JsResult<'a, JsArray> {
    let a = JsArray::new(cx, arr.len() as u32);

    for (i, s) in arr.iter().enumerate() {
        let v = cx.number(*s);
        a.set(cx, i as u32, v)?;
    }

    Ok(a)
}
