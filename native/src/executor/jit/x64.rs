use crate::component::{Instruction, TypeI, TypeJ, TypeR};
use crate::executor::error::ExecuteError;
use crate::executor::{Arch, Interpreter};
use crate::memory::Memory;
use dynasmrt::x64::Assembler;
use dynasmrt::{dynasm, AssemblyOffset, DynasmApi, ExecutableBuffer};
use rustc_hash::{FxHashMap, FxHashSet};
use std::mem;

type CompiledFunction = extern "win64" fn(&mut Arch, *mut u8);

#[derive(Debug)]
struct CompiledCode {
    offset: AssemblyOffset,
    buf: ExecutableBuffer,
}

#[derive(Debug)]
pub struct Jit {
    interpreter: Interpreter,
    codes: FxHashMap<u32, CompiledCode>,
    failures: FxHashSet<u32>,
}

impl Jit {
    pub fn new(mem: Box<dyn Memory>) -> Self {
        assert!(
            mem.fastmem_addr().is_some(),
            "JIT should be used only with fastmem"
        );

        Jit {
            interpreter: Interpreter::new(mem),
            codes: FxHashMap::default(),
            failures: FxHashSet::default(),
        }
    }

    pub fn as_arch(&self) -> &Arch {
        self.interpreter.as_arch()
    }

    pub fn as_arch_mut(&mut self) -> &mut Arch {
        self.interpreter.as_arch_mut()
    }

    pub fn into_arch(self) -> Arch {
        self.interpreter.into_arch()
    }

    pub fn step(&mut self) -> Result<(), ExecuteError> {
        self.interpreter.step()
    }

    pub fn exec(&mut self) -> Result<(), ExecuteError> {
        let addr_from = self.interpreter.as_arch().pc();

        let code = match self.codes.get(&addr_from) {
            Some(x) => x,
            None => {
                if self.failures.contains(&addr_from) {
                    return self.interpreter.step();
                }

                match self.compile(addr_from) {
                    Ok(x) => x,
                    Err(_) => {
                        self.failures.insert(addr_from);
                        return self.interpreter.step();
                    }
                }
            }
        };

        let f: CompiledFunction = unsafe { mem::transmute(code.buf.ptr(code.offset)) };
        let arch = self.interpreter.as_arch_mut();
        let base_addr = arch
            .mem
            .fastmem_addr()
            .expect("JIT should be used only with fastmem")
            .as_ptr();

        debug_assert_eq!(arch.reg[0], 0, "$0 should stay 0");
        arch.reg[0] = 0;

        f(arch, base_addr);

        debug_assert_eq!(arch.reg[0], 0, "JIT code modified $0");
        arch.reg[0] = 0;

        Ok(())
    }

    pub fn invalidate(&mut self) {
        self.codes.clear();
    }

    fn compile(&mut self, addr_from: u32) -> Result<&CompiledCode, ()> {
        use Instruction::*;

        let mem = self.interpreter.as_arch().mem();

        let mut should_set_pc = true;
        let mut addr = addr_from;
        let mut ops = Assembler::new().unwrap();
        let label = ops.offset();

        emit_prologue(&mut ops);

        // Encode at maximum 1000 instructions
        for i in 0..1000 {
            let ins_code = mem.read_u32(addr);
            let ins = Instruction::decode(ins_code);
            addr = addr.wrapping_add(4);

            match ins {
                add(x) => emit_add(&mut ops, x),
                addu(x) => emit_add(&mut ops, x),
                and(x) => emit_and(&mut ops, x),
                nor(x) => emit_nor(&mut ops, x),
                or(x) => emit_or(&mut ops, x),
                slt(x) => emit_slt(&mut ops, x),
                sltu(x) => emit_sltu(&mut ops, x),
                sub(x) => emit_sub(&mut ops, x),
                subu(x) => emit_sub(&mut ops, x),
                xor(x) => emit_xor(&mut ops, x),
                sll(x) => emit_sll(&mut ops, x),
                sllv(x) => emit_sllv(&mut ops, x),
                sra(x) => emit_sra(&mut ops, x),
                srav(x) => emit_srav(&mut ops, x),
                srl(x) => emit_srl(&mut ops, x),
                srlv(x) => emit_srlv(&mut ops, x),
                addi(x) => emit_addi(&mut ops, x),
                addiu(x) => emit_addi(&mut ops, x),
                andi(x) => emit_andi(&mut ops, x),
                lui(x) => emit_lui(&mut ops, x),
                ori(x) => emit_ori(&mut ops, x),
                slti(x) => emit_slti(&mut ops, x),
                sltiu(x) => emit_sltiu(&mut ops, x),
                xori(x) => emit_xori(&mut ops, x),
                lb(x) => emit_lb(&mut ops, x),
                lbu(x) => emit_lbu(&mut ops, x),
                lh(x) => emit_lh(&mut ops, x),
                lhu(x) => emit_lhu(&mut ops, x),
                lw(x) => emit_lw(&mut ops, x),
                sb(x) => emit_sb(&mut ops, x),
                sh(x) => emit_sh(&mut ops, x),
                sw(x) => emit_sw(&mut ops, x),
                j(x) => {
                    emit_j(&mut ops, x, addr);
                    should_set_pc = false;
                    break; // basic block finished
                }
                jal(x) => {
                    emit_jal(&mut ops, x, addr);
                    should_set_pc = false;
                    break; // basic block finished
                }
                jalr(x) => {
                    emit_jalr(&mut ops, x, addr);
                    should_set_pc = false;
                    break; // basic block finished
                }
                jr(x) => {
                    emit_jr(&mut ops, x);
                    should_set_pc = false;
                    break; //basic block finished
                }
                _ => {
                    // unsupported instruction (branches and syscall)
                    if i == 0 {
                        return Err(());
                    } else {
                        // we did not consume this instruction; rewind pc
                        addr = addr.wrapping_sub(4);
                        break;
                    }
                }
            }
        }

        emit_epilogue(&mut ops, if should_set_pc { Some(addr) } else { None });

        let buf = ops.finalize().unwrap();

        let code = CompiledCode { offset: label, buf };
        Ok(self.codes.entry(addr_from).or_insert(code))
    }
}

fn emit_prologue(_ops: &mut Assembler) {
    // do nothing
}

fn emit_epilogue(ops: &mut Assembler, pc: Option<u32>) {
    if let Some(addr) = pc {
        dynasm!(ops
            ; mov DWORD [rcx + 32*4], addr as _
        );
    }

    dynasm!(ops
        ; ret
    );
}

fn emit_add(ops: &mut Assembler, x: TypeR) {
    if x.rd.is_zero() {
        return;
    }

    //TODO: Implement overflow exception
    dynasm!(ops
        ; mov eax, DWORD [rcx + (x.rs.num() as i32) * 4]
        ; add eax, DWORD [rcx + (x.rt.num() as i32) * 4]
        ; mov DWORD [rcx + (x.rd.num() as i32) * 4], eax
    );
}

fn emit_and(ops: &mut Assembler, x: TypeR) {
    if x.rd.is_zero() {
        return;
    }

    dynasm!(ops
        ; mov eax, DWORD [rcx + (x.rs.num() as i32) * 4]
        ; and eax, DWORD [rcx + (x.rt.num() as i32) * 4]
        ; mov DWORD [rcx + (x.rd.num() as i32) * 4], eax
    );
}

fn emit_nor(ops: &mut Assembler, x: TypeR) {
    if x.rd.is_zero() {
        return;
    }

    dynasm!(ops
        ; mov eax, DWORD [rcx + (x.rs.num() as i32) * 4]
        ; or eax, DWORD [rcx + (x.rt.num() as i32) * 4]
        ; not eax
        ; mov DWORD [rcx + (x.rd.num() as i32) * 4], eax
    );
}

fn emit_or(ops: &mut Assembler, x: TypeR) {
    if x.rd.is_zero() {
        return;
    }

    dynasm!(ops
        ; mov eax, DWORD [rcx + (x.rs.num() as i32) * 4]
        ; or eax, DWORD [rcx + (x.rt.num() as i32) * 4]
        ; mov DWORD [rcx + (x.rd.num() as i32) * 4], eax
    );
}

fn emit_slt(ops: &mut Assembler, x: TypeR) {
    if x.rd.is_zero() {
        return;
    }

    dynasm!(ops
        ; mov eax, DWORD [rcx + (x.rt.num() as i32) * 4]
        ; cmp DWORD [rcx + (x.rs.num() as i32) * 4], eax
        ; setl al
        ; movzx eax, al
        ; mov DWORD [rcx + (x.rd.num() as i32) * 4], eax
    );
}

fn emit_sltu(ops: &mut Assembler, x: TypeR) {
    if x.rd.is_zero() {
        return;
    }

    dynasm!(ops
        ; mov eax, DWORD [rcx + (x.rt.num() as i32) * 4]
        ; cmp DWORD [rcx + (x.rs.num() as i32) * 4], eax
        ; setb al
        ; movzx eax, al
        ; mov DWORD [rcx + (x.rd.num() as i32) * 4], eax
    );
}

fn emit_sub(ops: &mut Assembler, x: TypeR) {
    if x.rd.is_zero() {
        return;
    }

    dynasm!(ops
        ; mov eax, DWORD [rcx + (x.rs.num() as i32) * 4]
        ; sub eax, DWORD [rcx + (x.rt.num() as i32) * 4]
        ; mov DWORD [rcx + (x.rd.num() as i32) * 4], eax
    );
}

fn emit_xor(ops: &mut Assembler, x: TypeR) {
    if x.rd.is_zero() {
        return;
    }

    dynasm!(ops
        ; mov eax, DWORD [rcx + (x.rs.num() as i32) * 4]
        ; xor eax, DWORD [rcx + (x.rt.num() as i32) * 4]
        ; mov DWORD [rcx + (x.rd.num() as i32) * 4], eax
    );
}

fn emit_sll(ops: &mut Assembler, x: TypeR) {
    if x.rd.is_zero() {
        return;
    }

    dynasm!(ops
        ; mov eax, DWORD [rcx + (x.rt.num() as i32) * 4]
        ; sal eax, x.shamt as i8
        ; mov DWORD [rcx + (x.rd.num() as i32) * 4], eax
    );
}

fn emit_sllv(ops: &mut Assembler, x: TypeR) {
    if x.rd.is_zero() {
        return;
    }

    dynasm!(ops
        ; mov r8, rcx
        ; mov eax, DWORD [rcx + (x.rt.num() as i32) * 4]
        ; mov ecx, DWORD [rcx + (x.rs.num() as i32) * 4]
        ; sal eax, cl
        ; mov DWORD [r8 + (x.rd.num() as i32) * 4], eax
        ; mov rcx, r8
    );
}

fn emit_sra(ops: &mut Assembler, x: TypeR) {
    if x.rd.is_zero() {
        return;
    }

    dynasm!(ops
        ; mov eax, DWORD [rcx + (x.rt.num() as i32) * 4]
        ; sar eax, x.shamt as i8
        ; mov DWORD [rcx + (x.rd.num() as i32) * 4], eax
    );
}

fn emit_srav(ops: &mut Assembler, x: TypeR) {
    if x.rd.is_zero() {
        return;
    }

    dynasm!(ops
        ; mov r8, rcx
        ; mov eax, DWORD [rcx + (x.rt.num() as i32) * 4]
        ; mov ecx, DWORD [rcx + (x.rs.num() as i32) * 4]
        ; sar eax, cl
        ; mov DWORD [r8 + (x.rd.num() as i32) * 4], eax
        ; mov rcx, r8
    );
}

fn emit_srl(ops: &mut Assembler, x: TypeR) {
    if x.rd.is_zero() {
        return;
    }

    dynasm!(ops
        ; mov eax, DWORD [rcx + (x.rt.num() as i32) * 4]
        ; shr eax, x.shamt as i8
        ; mov DWORD [rcx + (x.rd.num() as i32) * 4], eax
    );
}

fn emit_srlv(ops: &mut Assembler, x: TypeR) {
    if x.rd.is_zero() {
        return;
    }

    dynasm!(ops
        ; mov r8, rcx
        ; mov eax, DWORD [rcx + (x.rt.num() as i32) * 4]
        ; mov ecx, DWORD [rcx + (x.rs.num() as i32) * 4]
        ; shr eax, cl
        ; mov DWORD [r8 + (x.rd.num() as i32) * 4], eax
        ; mov rcx, r8
    );
}

fn emit_addi(ops: &mut Assembler, x: TypeI) {
    if x.rt.is_zero() {
        return;
    }

    dynasm!(ops
        ; mov eax, DWORD [rcx + (x.rs.num() as i32) * 4]
        ; add eax, x.imm as i16 as i32
        ; mov DWORD [rcx + (x.rt.num() as i32) * 4], eax
    );
}

fn emit_andi(ops: &mut Assembler, x: TypeI) {
    if x.rt.is_zero() {
        return;
    }

    dynasm!(ops
        ; mov eax, DWORD [rcx + (x.rs.num() as i32) * 4]
        ; and eax, x.imm as u32 as i32
        ; mov DWORD [rcx + (x.rt.num() as i32) * 4], eax
    );
}

fn emit_lui(ops: &mut Assembler, x: TypeI) {
    if x.rt.is_zero() {
        return;
    }

    dynasm!(ops
        ; mov DWORD [rcx + (x.rt.num() as i32) * 4], ((x.imm as u32) << 16) as i32
    );
}

fn emit_ori(ops: &mut Assembler, x: TypeI) {
    if x.rt.is_zero() {
        return;
    }

    dynasm!(ops
        ; mov eax, DWORD [rcx + (x.rs.num() as i32) * 4]
        ; or eax, x.imm as u32 as i32
        ; mov DWORD [rcx + (x.rt.num() as i32) * 4], eax
    );
}

fn emit_slti(ops: &mut Assembler, x: TypeI) {
    if x.rt.is_zero() {
        return;
    }

    dynasm!(ops
        ; xor eax, eax
        ; cmp DWORD [rcx + (x.rs.num() as i32) * 4], (x.imm as i16 as i32) - 1
        ; setle al
        ; mov DWORD [rcx + (x.rt.num() as i32) * 4], eax
    );
}

fn emit_sltiu(ops: &mut Assembler, x: TypeI) {
    if x.rt.is_zero() {
        return;
    }

    dynasm!(ops
        ; xor eax, eax
        ; cmp DWORD [rcx + (x.rs.num() as i32) * 4], (x.imm as i16 as i32) - 1
        ; setbe al
        ; mov DWORD [rcx + (x.rt.num() as i32) * 4], eax
    );
}

fn emit_xori(ops: &mut Assembler, x: TypeI) {
    if x.rt.is_zero() {
        return;
    }

    dynasm!(ops
        ; mov eax, DWORD [rcx + (x.rs.num() as i32) * 4]
        ; xor eax, x.imm as u32 as i32
        ; mov DWORD [rcx + (x.rt.num() as i32) * 4], eax
    );
}

fn emit_lb(ops: &mut Assembler, x: TypeI) {
    if x.rt.is_zero() {
        return;
    }

    dynasm!(ops
        ; mov eax, DWORD [rcx + (x.rs.num() as i32) * 4]
        ; add eax, x.imm as i16 as i32
        ; movsx eax, BYTE [rdx + rax]
        ; mov DWORD [rcx + (x.rt.num() as i32) * 4], eax
    );
}

fn emit_lbu(ops: &mut Assembler, x: TypeI) {
    if x.rt.is_zero() {
        return;
    }

    dynasm!(ops
        ; mov eax, DWORD [rcx + (x.rs.num() as i32) * 4]
        ; add eax, x.imm as i16 as i32
        ; movzx eax, BYTE [rdx + rax]
        ; mov DWORD [rcx + (x.rt.num() as i32) * 4], eax
    );
}

fn emit_lh(ops: &mut Assembler, x: TypeI) {
    if x.rt.is_zero() {
        return;
    }

    dynasm!(ops
        ; mov eax, DWORD [rcx + (x.rs.num() as i32) * 4]
        ; add eax, x.imm as i16 as i32
        ; movsx eax, WORD [rdx + rax]
        ; mov DWORD [rcx + (x.rt.num() as i32) * 4], eax
    );
}

fn emit_lhu(ops: &mut Assembler, x: TypeI) {
    if x.rt.is_zero() {
        return;
    }

    dynasm!(ops
        ; mov eax, DWORD [rcx + (x.rs.num() as i32) * 4]
        ; add eax, x.imm as i16 as i32
        ; movzx eax, WORD [rdx + rax]
        ; mov DWORD [rcx + (x.rt.num() as i32) * 4], eax
    );
}

fn emit_lw(ops: &mut Assembler, x: TypeI) {
    if x.rt.is_zero() {
        return;
    }

    dynasm!(ops
        ; mov eax, DWORD [rcx + (x.rs.num() as i32) * 4]
        ; add eax, x.imm as i16 as i32
        ; mov eax, DWORD [rdx + rax]
        ; mov DWORD [rcx + (x.rt.num() as i32) * 4], eax
    );
}

fn emit_sb(ops: &mut Assembler, x: TypeI) {
    dynasm!(ops
        ; mov eax, DWORD [rcx + (x.rs.num() as i32) * 4]
        ; mov r8d, DWORD [rcx + (x.rt.num() as i32) * 4]
        ; add eax, x.imm as i16 as i32
        ; mov BYTE [rdx + rax], r8b
    );
}

fn emit_sh(ops: &mut Assembler, x: TypeI) {
    dynasm!(ops
        ; mov eax, DWORD [rcx + (x.rs.num() as i32) * 4]
        ; mov r8d, DWORD [rcx + (x.rt.num() as i32) * 4]
        ; add eax, x.imm as i16 as i32
        ; mov WORD [rdx + rax], r8w
    );
}

fn emit_sw(ops: &mut Assembler, x: TypeI) {
    dynasm!(ops
        ; mov eax, DWORD [rcx + (x.rs.num() as i32) * 4]
        ; mov r8d, DWORD [rcx + (x.rt.num() as i32) * 4]
        ; add eax, x.imm as i16 as i32
        ; mov DWORD [rdx + rax], r8d
    );
}

fn emit_j(ops: &mut Assembler, x: TypeJ, pc: u32) {
    dynasm!(ops
        ; mov eax, (pc & 0xf000_0000) as i32
        ; or eax, (x.target << 2) as i32
        ; mov DWORD [rcx + 32*4], eax
    );
}

fn emit_jal(ops: &mut Assembler, x: TypeJ, pc: u32) {
    emit_j(ops, x, pc);
    dynasm!(ops
        ; mov DWORD [rcx + 31 * 4], pc as i32
    );
}

fn emit_jalr(ops: &mut Assembler, x: TypeR, pc: u32) {
    emit_jr(ops, x);
    dynasm!(ops
        ; mov DWORD [rcx + (x.rd.num() as i32) * 4], pc as i32
    );
}

fn emit_jr(ops: &mut Assembler, x: TypeR) {
    dynasm!(ops
        ; mov eax, DWORD [rcx + (x.rs.num() as i32) * 4]
        ; mov DWORD [rcx + 32*4], eax
    );
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::assembler::assemble;
    use crate::memory::{create_memory_fastmem, EndianMode};
    use lazy_static::lazy_static;
    use parking_lot::Mutex;

    // To prevent exhausting address space
    lazy_static! {
        static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
    }

    fn init_state(asm: &str) -> Jit {
        let segs = assemble(EndianMode::native(), asm).unwrap();
        Jit::new(create_memory_fastmem(EndianMode::native(), &segs))
    }

    #[test]
    fn add() {
        let _guard = TEST_MUTEX.lock();
        let mut jit = init_state(".text\nadd $16, $17, $18\nsyscall");

        jit.as_arch_mut().reg[16] = 0;
        jit.as_arch_mut().reg[17] = 1;
        jit.as_arch_mut().reg[18] = 2;

        jit.exec().unwrap();

        assert!(jit.codes.contains_key(&0x0040_0024));
        assert_eq!(jit.as_arch_mut().pc(), 0x0040_0028);
        assert_eq!(jit.as_arch_mut().reg[16], 3);
    }

    #[test]
    fn sub() {
        let _guard = TEST_MUTEX.lock();
        let mut jit = init_state(".text\nsub $16, $17, $18\nsyscall");

        jit.as_arch_mut().reg[16] = 0;
        jit.as_arch_mut().reg[17] = 1;
        jit.as_arch_mut().reg[18] = 2;

        jit.exec().unwrap();

        assert!(jit.codes.contains_key(&0x0040_0024));
        assert_eq!(jit.as_arch_mut().pc(), 0x0040_0028);
        assert_eq!(jit.as_arch_mut().reg[16], -1_i32 as u32);
    }

    #[test]
    fn sllv() {
        let _guard = TEST_MUTEX.lock();
        let mut jit = init_state(".text\nsllv $16, $17, $18\nsyscall");

        jit.as_arch_mut().reg[16] = 0;
        jit.as_arch_mut().reg[17] = 1234;
        jit.as_arch_mut().reg[18] = 2;

        jit.exec().unwrap();

        assert!(jit.codes.contains_key(&0x0040_0024));
        assert_eq!(jit.as_arch_mut().pc(), 0x0040_0028);
        assert_eq!(jit.as_arch_mut().reg[16], 1234 << 2);
    }

    #[test]
    fn lw() {
        let _guard = TEST_MUTEX.lock();
        let mut jit = init_state(".text\nlw $17, 0($16)\nlw $18, -1($16)\nlw $19, 1($16)\nsyscall");

        jit.as_arch_mut().reg[16] = 0x1000_0000;
        jit.as_arch_mut().mem.write_u32(0x1000_0000 - 4, 0x11223344);
        jit.as_arch_mut().mem.write_u32(0x1000_0000, 0x55667788);
        jit.as_arch_mut().mem.write_u32(0x1000_0000 + 4, 0x99aabbcc);

        let ans_1 = jit.as_arch_mut().mem.read_u32(0x1000_0000);
        let ans_2 = jit.as_arch_mut().mem.read_u32(0x1000_0000 - 1);
        let ans_3 = jit.as_arch_mut().mem.read_u32(0x1000_0000 + 1);

        jit.exec().unwrap();

        assert!(jit.codes.contains_key(&0x0040_0024));
        assert_eq!(jit.as_arch_mut().pc(), 0x0040_0030);
        assert_eq!(jit.as_arch_mut().reg[16], 0x1000_0000);
        assert_eq!(jit.as_arch_mut().reg[17], ans_1);
        assert_eq!(jit.as_arch_mut().reg[18], ans_2);
        assert_eq!(jit.as_arch_mut().reg[19], ans_3);
    }

    #[test]
    fn fibonacci() {
        let code = r"
# Recursive fibonacci calculator
# Function signature: int fibonacci(int)
# Also saves the result into $gp as an int array (e.g. $gp = fibonacci(2), $gp + 4 = fibonacci(3), ...)
# Modified from https://gist.github.com/libertylocked/068b118354539a8be992
.text
.globl main
main:
    # Calculate fibonacci upto this number (7)
    ori $a0, $0, 7
    or $s0, $ra, $zero
    jal fibonacci

    # Now we have the answer in $v0
    # NOP here so you can check out register pane
    add $0, $0, $0

    or $ra, $s0, $zero
    # Terminate the program
    jr $ra
fibonacci:
    # Prologue
    addi $sp, $sp, -12
    sw $ra, 8($sp)
    sw $s0, 4($sp)
    sw $s1, 0($sp)
    or $s0, $a0, $zero
    ori $v0, $zero, 1 # return value for terminal condition
    slti $t0, $16, 3
    bne $t0, $0, fibonacciExit # check terminal condition
    addi $a0, $s0, -1 # set args for recursive call to f(n-1)
    jal fibonacci
    or $s1, $v0, $zero # store result of f(n-1) to s1
    addi $a0, $s0, -2 # set args for recursive call to f(n-2)
    jal fibonacci
    add $v0, $s1, $v0 # add result of f(n-1) to it
fibonacciExit:
    # Save value to memory
    add $t0, $s0, $s0
    add $t0, $t0, $t0  # multiply by 4
    addi $t0, $t0, -8  # align that fibonacci(2) ==> $gp
    add $t0, $gp, $t0
    sw $v0, 0($t0)
    # Epilogue
    lw $ra, 8($sp)
    lw $s0, 4($sp)
    lw $s1, 0($sp)
    addi $sp, $sp, 12
    jr $ra";

        let mut state = init_state(code);

        for i in 0..1000000 {
            let pc = state.as_arch().pc();
            if pc == 0 {
                break;
            }

            state.exec().unwrap();
        }

        assert_eq!(state.as_arch().pc(), 0);

        let data_addr = 0x1000_8000;
        assert_eq!(state.as_arch_mut().mem.read_u32(data_addr), 1);
        assert_eq!(state.as_arch_mut().mem.read_u32(data_addr + 4), 2);
        assert_eq!(state.as_arch_mut().mem.read_u32(data_addr + 8), 3);
        assert_eq!(state.as_arch_mut().mem.read_u32(data_addr + 12), 5);
        assert_eq!(state.as_arch_mut().mem.read_u32(data_addr + 16), 8);
        assert_eq!(state.as_arch_mut().mem.read_u32(data_addr + 20), 13);
        assert_eq!(state.as_arch_mut().mem.read_u32(data_addr + 24), 0);
    }
}
