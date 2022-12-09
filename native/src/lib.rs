mod architecture;
mod assembler;
mod component;
mod memory;
mod webapi;
mod interpreter;

use neon::prelude::*;

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    webapi::register_functions(&mut cx)?;
    Ok(())
}
