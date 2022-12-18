use crate::assembler::assemble;
use crate::component::RegisterName;
use crate::disassembler::disassemble;
use crate::executor::{Executor, Interpreter, Jit, Pipeline, HAS_JIT};
use crate::memory::{create_empty_memory, create_memory, EndianMode};
use crate::webapi::updates::Updates;
use neon::prelude::*;
use parking_lot::Mutex;
use rustc_hash::FxHashMap;
use std::mem::swap;
use std::ops::RangeInclusive;
use std::sync::Arc;

#[derive(Debug)]
pub struct State {
    channel: Channel,
    callback: Arc<Root<JsFunction>>,
    inner: Inner,
}

#[derive(Debug)]
struct Inner {
    clean_after_reset: bool,
    exec: Executor,
    disassembly_range: Mutex<Option<RangeInclusive<u32>>>,
}

impl Default for Inner {
    fn default() -> Self {
        let interpreter = Interpreter::new(create_empty_memory(EndianMode::native()));

        Inner {
            clean_after_reset: true,
            exec: Executor::ExInterpreter(interpreter),
            disassembly_range: Mutex::new(None),
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
        ret.notify(Updates::all());
        ret
    }

    pub fn reset(&mut self) -> Updates {
        self.inner = Default::default();
        Updates::all()
    }

    pub fn assemble(&mut self, code: &str, endian: EndianMode) -> Result<Updates, String> {
        let segs = assemble(endian, code).map_err(|e| e.to_string())?;
        let mem = create_memory(endian, &segs);

        if HAS_JIT && endian == EndianMode::native() {
            self.inner.exec = Executor::ExJit(Jit::new(mem));
        } else {
            self.inner.exec = Executor::ExInterpreter(Interpreter::new(mem));
        }

        Ok(Updates::all())
    }

    pub fn edit_register(&mut self, r: RegisterName, val: u32) -> Updates {
        self.inner.clean_after_reset = false;
        self.inner.exec.as_arch_mut().set_reg(r, val);
        Updates::REGISTERS
    }

    pub fn read_memory(&self, page_idx: u32, output: &mut [u8]) {
        let addr = page_idx * 4096;
        let mem = self.inner.exec.as_arch().mem();
        mem.read_into_slice(addr, output);
    }

    pub fn step(&mut self) -> Result<Updates, String> {
        self.inner.clean_after_reset = false;
        if self.inner.exec.as_arch().pc() < 0x00001000 {
            Ok(Updates::empty())
        } else {
            self.inner.exec.step().map_err(|x| format!("{:?}", x))?;
            Ok(Updates::REGISTERS)
        }
    }

    pub fn exec(&mut self) -> Result<Updates, String> {
        self.inner.clean_after_reset = false;
        if self.inner.exec.as_arch().pc() < 0x00001000 {
            Ok(Updates::empty())
        } else {
            self.inner.exec.exec().map_err(|x| format!("{:?}", x))?;
            Ok(Updates::REGISTERS)
        }
    }

    pub fn run(&self, allow_jit: bool) -> Updates {
        super::looper::start(allow_jit);
        Updates::FLAG_RUNNING
    }

    pub fn stop(&self) -> Updates {
        if super::looper::stop() {
            Updates::all()
        } else {
            Updates::FLAG_RUNNING
        }
    }

    pub fn convert_to_pipeline(&mut self) -> Updates {
        if self.inner.capture_can_use_pipeline() {
            return Updates::empty();
        }

        let endian = self.inner.exec.as_arch().mem().endian();
        let mut pipeline = Executor::ExPipeline(Pipeline::new(create_empty_memory(endian)));
        swap(&mut pipeline, &mut self.inner.exec);

        *self.inner.exec.as_arch_mut() = pipeline.into_arch();

        Updates::all()
    }

    pub fn notify(&self, mut updates: Updates) {
        let callback = self.callback.clone();

        if self.inner.needs_capture_disasm() {
            updates |= Updates::DISASSEMBLY;
        }

        // cheap-to-collect ones
        let clean_after_reset = self.inner.clean_after_reset;
        let running = self.inner.capture_running();
        let can_use_jit = self.inner.capture_can_use_jit();
        let can_use_pipeline = self.inner.capture_can_use_pipeline();
        let pc = self.inner.capture_pc();

        // expensive-to-collect ones
        let regs = if updates.contains(Updates::REGISTERS) {
            self.inner.capture_regs()
        } else {
            [0; 32]
        };

        let pipeline_detail = if updates.contains(Updates::REGISTERS) {
            Some(self.inner.capture_pipeline_detail())
        } else {
            None
        };

        let disasm_mapping = if updates.contains(Updates::DISASSEMBLY) {
            self.inner.capture_disasm()
        } else {
            FxHashMap::default()
        };

        // send the notification
        self.channel.send(move |mut cx| {
            let obj = cx.empty_object();

            if updates.contains(Updates::REGISTERS) {
                let regs = js_array_numbers(&mut cx, regs.iter())?;
                let pc = cx.number(pc);
                obj.set(&mut cx, "regs", regs)?;
                obj.set(&mut cx, "pc", pc)?;

                if let Some(x) = pipeline_detail {
                    let str = cx.string(x);
                    obj.set(&mut cx, "pipelineDetail", str)?;
                }
            }

            if updates.contains(Updates::DISASSEMBLY) {
                let disasm = cx.empty_object();
                for (k, v) in disasm_mapping.iter() {
                    let number = cx.number(v.0);
                    let value = cx.string(&v.1);
                    let tuple = cx.empty_array();
                    tuple.set(&mut cx, 0, number)?;
                    tuple.set(&mut cx, 1, value)?;
                    disasm.set(&mut cx, *k, tuple)?;
                }

                let mut disasm_list = disasm_mapping.keys().copied().collect::<Vec<u32>>();
                disasm_list.sort();
                let disasm_list = js_array_numbers(&mut cx, disasm_list.iter())?;

                obj.set(&mut cx, "disasm", disasm)?;
                obj.set(&mut cx, "disasmList", disasm_list)?;
            }

            if updates.contains(Updates::FLAG_RUNNING) {
                let running = cx.boolean(running);
                obj.set(&mut cx, "running", running)?;
            }

            if updates.contains(Updates::FLAG_CAN_USE_JIT) {
                let can_use_jit = cx.boolean(can_use_jit);
                obj.set(&mut cx, "canUseJit", can_use_jit)?;
            }

            /* unconditional updates */
            {
                let clean_after_reset = cx.boolean(clean_after_reset);
                let can_use_pipeline = cx.boolean(can_use_pipeline);
                obj.set(&mut cx, "cleanAfterReset", clean_after_reset)?;
                obj.set(&mut cx, "canUsePipeline", can_use_pipeline)?;
            }

            callback
                .to_inner(&mut cx)
                .call_with(&cx)
                .arg(obj)
                .exec(&mut cx)
        });
    }
}

impl Inner {
    fn needs_capture_disasm(&self) -> bool {
        let range = self.disassembly_range.lock();

        match range.as_ref() {
            Some(x) => !x.contains(&self.exec.as_arch().pc()),
            None => true,
        }
    }

    fn capture_regs(&self) -> [u32; 32] {
        let mut ret = [0; 32];
        self.exec.as_arch().read_all_reg(&mut ret);
        ret
    }

    fn capture_pc(&self) -> u32 {
        self.exec.as_arch().pc()
    }

    fn capture_disasm(&self) -> FxHashMap<u32, (u32, String)> {
        let mut range = self.disassembly_range.lock();
        let pc = self.exec.as_arch().pc();
        let mem = self.exec.as_arch().mem();
        let mut mapping = FxHashMap::default();
        let mut min_addr = pc;
        let mut max_addr = pc;

        /* walk back */
        {
            let mut addr = pc - 4;
            let mut nop_cnt: u32 = 0;
            while addr > 4096 && nop_cnt < 16 {
                let x = mem.read_u32(addr);

                if x == 0x00000000 || x == 0x00000020 {
                    nop_cnt += 1;
                } else {
                    nop_cnt = 0;
                }

                mapping.insert(addr, (x, disassemble(x)));
                min_addr = addr;
                addr -= 4;
            }
        }

        /* walk forward */
        {
            let mut addr = pc & (!0xfff);
            let mut nop_cnt: u32 = 0;
            while addr < 0x1000_0000 && nop_cnt < 256 {
                let x = mem.read_u32(addr);

                if x == 0x00000000 || x == 0x00000020 {
                    nop_cnt += 1;
                } else {
                    nop_cnt = 0;
                }

                mapping.insert(addr, (x, disassemble(x)));
                max_addr = addr;
                addr += 4;
            }
        }

        *range = Some(min_addr..=max_addr);

        mapping
    }

    fn capture_pipeline_detail(&self) -> String {
        let detail = if let Executor::ExPipeline(x) = &self.exec {
            x.get_pipeline_detail()
        } else {
            Default::default()
        };

        serde_json::to_string(&detail).unwrap_or_else(|_| "".into())
    }

    fn capture_running(&self) -> bool {
        super::looper::is_running()
    }

    fn capture_can_use_jit(&self) -> bool {
        match self.exec {
            Executor::ExInterpreter(_) => false,
            Executor::ExJit(_) => true,
            Executor::ExPipeline(_) => false,
        }
    }

    fn capture_can_use_pipeline(&self) -> bool {
        match self.exec {
            Executor::ExInterpreter(_) => false,
            Executor::ExJit(_) => false,
            Executor::ExPipeline(_) => true,
        }
    }
}

fn js_array_numbers<'a, 'b, C: Context<'a>>(
    cx: &mut C,
    iter: impl Iterator<Item = &'b u32>,
) -> JsResult<'a, JsArray> {
    let size_hint = iter.size_hint();
    let len = size_hint.1.unwrap_or(size_hint.0);
    let a = JsArray::new(cx, len as u32);

    for (i, s) in iter.enumerate() {
        let v = cx.number(*s);
        a.set(cx, i as u32, v)?;
    }

    Ok(a)
}
