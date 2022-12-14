mod assembler;
mod component;
mod disassembler;
mod executor;
mod memory;
mod webapi;

use neon::prelude::*;

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    webapi::register_functions(&mut cx)?;
    Ok(())
}
