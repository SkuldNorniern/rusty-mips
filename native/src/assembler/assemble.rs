use super::error::*;
use crate::component::{Instruction, RegisterName, TypeI, TypeJ, TypeR};
use crate::memory::{EndianMode, Segment};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::ops::RangeInclusive;
use std::str::FromStr;

/*
Following instructions are well supported:
and, or, add, sub, slt, lw, sw, beq, j

Most other MIPS-1 instructions should be supported, but there might be bugs.
Some instructions like break are not supported.

Syntax is like SPIM simulator.
Note: https://phoenix.goucher.edu/~kelliher/f2009/cs220/mipsir.html
*/

lazy_static! {
    static ref RE_SEPARATOR: Regex = Regex::new(r"[\s,]+").unwrap();
}

enum Token<'a> {
    Text { text: &'a str },
    Number { text: &'a str, num: i64 },
    Register { text: &'a str, reg: RegisterName },
    LabelDef { text: &'a str },
}

impl Token<'_> {
    fn as_text(&self) -> &str {
        match self {
            Token::Text { text } => text,
            Token::Number { text, .. } => text,
            Token::Register { text, .. } => text,
            Token::LabelDef { text } => text,
        }
    }

    fn as_number(&self) -> Result<i64, AssemblerError> {
        if let Token::Number { num, .. } = self {
            Ok(*num)
        } else {
            TokenNotNumberSnafu {
                token: self.as_text(),
            }
            .fail()
        }
    }

    fn as_register(&self) -> Result<RegisterName, AssemblerError> {
        if let Token::Register { reg, .. } = self {
            Ok(*reg)
        } else {
            TokenNotRegisterSnafu {
                token: self.as_text(),
            }
            .fail()
        }
    }
}

struct LineContext<'a> {
    line: &'a str,
    mnemonic: &'a str,
    args: &'a [Token<'a>],
    args_raw: &'a str,
    pc: u32,
    labels: &'a Option<HashMap<String, u32>>,
}

impl LineContext<'_> {
    fn resolve_label(&self, label: &Token, relative: bool) -> Result<u32, AssemblerError> {
        let next_pc = self.pc.wrapping_add(4);

        let target = if let Token::Number { num, .. } = label {
            if relative {
                // parse as offset
                let offset: i32 = (*num)
                    .try_into()
                    .map_err(|_| ImmediateTooLargeSnafu { imm: *num }.build())?;
                next_pc.wrapping_add(offset as u32)
            } else {
                // parse as target address
                (*num)
                    .try_into()
                    .map_err(|_| ImmediateTooLargeSnafu { imm: *num }.build())?
            }
        } else if self.labels.is_some() {
            // parse as label
            *self
                .labels
                .as_ref()
                .unwrap()
                .get(label.as_text())
                .ok_or_else(|| {
                    LabelNotFoundSnafu {
                        label: label.as_text(),
                    }
                    .build()
                })?
        } else {
            // parse as label, but it's first pass; treat as 0
            self.pc
        };

        if target % 4 != 0 {
            JumpTargetUnalignedSnafu { target }.fail()?;
        }

        Ok(target)
    }

    fn resolve_branch(&self, label: &Token) -> Result<i16, AssemblerError> {
        let target = self.resolve_label(label, true)?;
        let encoded = target.wrapping_sub(self.pc.wrapping_add(4)) as i32 / 4;
        expect_extendable(encoded as _, true)?;
        Ok(encoded as i16)
    }

    fn resolve_jump(&self, label: &Token) -> Result<u32, AssemblerError> {
        let target = self.resolve_label(label, false)?;

        if target & 0xF000_0000 != self.pc.wrapping_add(4) & 0xF000_0000 {
            return JumpTooFarSnafu {
                target,
                pc: self.pc,
            }
            .fail();
        }

        Ok((target / 4) & 0x03FF_FFFF)
    }
}

fn tokenize<'a>(args: impl Iterator<Item = &'a str>) -> Vec<Token<'a>> {
    args.filter(|x| !x.is_empty())
        .map(|token| {
            if let Some(x) = try_parse_reg(token) {
                Token::Register {
                    text: token,
                    reg: x,
                }
            } else if let Some(x) = try_parse_number(token) {
                Token::Number {
                    text: token,
                    num: x,
                }
            } else if token.ends_with(':') {
                Token::LabelDef { text: token }
            } else {
                Token::Text { text: token }
            }
        })
        .collect()
}

fn try_parse_number(text: &str) -> Option<i64> {
    let text = text.to_ascii_lowercase();

    if let Some(x) = text.strip_prefix("0x") {
        i64::from_str_radix(x, 16).ok()
    } else if let Some(x) = text.strip_prefix("0o") {
        i64::from_str_radix(x, 8).ok()
    } else if let Some(x) = text.strip_prefix("0b") {
        i64::from_str_radix(x, 2).ok()
    } else {
        i64::from_str(&text).ok()
    }
}

fn try_parse_reg(name: &str) -> Option<RegisterName> {
    name.strip_prefix('$').and_then(RegisterName::try_from_name)
}

fn expect_args_count(ctx: &LineContext<'_>, expect_len: usize) -> Result<(), AssemblerError> {
    if ctx.args.len() == expect_len {
        Ok(())
    } else {
        InvalidNumberOfOperandsSnafu { line: ctx.line }.fail()
    }
}

fn expect_extendable(val: i64, sign_ext: bool) -> Result<u16, AssemblerError> {
    let interpreted = if sign_ext {
        val as i16 as i64
    } else {
        val as u16 as u64 as i64
    };

    if interpreted == val {
        Ok(val as u16)
    } else {
        ImmediateTooLargeSnafu { imm: val }.fail()
    }
}

fn try_parse_ins_3arg(ctx: &mut LineContext<'_>) -> Result<TypeR, AssemblerError> {
    expect_args_count(ctx, 3)?;

    Ok(TypeR {
        rd: ctx.args[0].as_register()?,
        rs: ctx.args[1].as_register()?,
        rt: ctx.args[2].as_register()?,
        shamt: 0,
    })
}

fn try_parse_ins_shift_reg(ctx: &mut LineContext<'_>) -> Result<TypeR, AssemblerError> {
    expect_args_count(ctx, 3)?;

    // Shift instructions use $d, $t, $s order,
    // where normal ones use $d, $s, $t order.
    Ok(TypeR {
        rd: ctx.args[0].as_register()?,
        rt: ctx.args[1].as_register()?,
        rs: ctx.args[2].as_register()?,
        shamt: 0,
    })
}

fn try_parse_ins_imm(ctx: &mut LineContext<'_>, sign_ext: bool) -> Result<TypeI, AssemblerError> {
    expect_args_count(ctx, 3)?;

    Ok(TypeI {
        rt: ctx.args[0].as_register()?,
        rs: ctx.args[1].as_register()?,
        imm: expect_extendable(ctx.args[2].as_number()?, sign_ext)?,
    })
}

fn try_parse_ins_lui(ctx: &mut LineContext<'_>) -> Result<TypeI, AssemblerError> {
    expect_args_count(ctx, 2)?;

    Ok(TypeI {
        rt: ctx.args[0].as_register()?,
        rs: RegisterName::new(0),
        imm: expect_extendable(ctx.args[1].as_number()?, false)?,
    })
}

fn try_parse_ins_shift_imm(ctx: &mut LineContext<'_>) -> Result<TypeR, AssemblerError> {
    expect_args_count(ctx, 3)?;

    let rd = ctx.args[0].as_register()?;
    let rt = ctx.args[1].as_register()?;
    let imm = ctx.args[2].as_number()?;

    if 32 <= imm {
        return ImmediateTooLargeSnafu { imm: imm as i64 }.fail();
    }

    Ok(TypeR {
        rs: RegisterName::new(0),
        rt,
        rd,
        shamt: imm as u8,
    })
}

fn try_parse_ins_memory(ctx: &mut LineContext<'_>) -> Result<TypeI, AssemblerError> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(\$.+)\s*,\s*([^ ]+?)\((\$.+)\)").unwrap();
    }

    let caps = match RE.captures(ctx.args_raw) {
        Some(x) => x,
        None => InvalidNumberOfOperandsSnafu { line: ctx.line }.fail()?,
    };

    let imm = expect_extendable(
        try_parse_number(&caps[2]).ok_or_else(|| InvalidTokenSnafu { token: &caps[2] }.build())?,
        true,
    )?;
    let rs =
        try_parse_reg(&caps[3]).ok_or_else(|| InvalidTokenSnafu { token: &caps[2] }.build())?;
    let rt =
        try_parse_reg(&caps[1]).ok_or_else(|| InvalidTokenSnafu { token: &caps[2] }.build())?;

    Ok(TypeI { rs, rt, imm })
}

fn try_parse_ins_branch(ctx: &mut LineContext<'_>) -> Result<TypeI, AssemblerError> {
    expect_args_count(ctx, 3)?;

    Ok(TypeI {
        rs: ctx.args[0].as_register()?,
        rt: ctx.args[1].as_register()?,
        imm: ctx.resolve_branch(&ctx.args[2])? as u16,
    })
}

fn try_parse_ins_branch_complex(ctx: &mut LineContext) -> Result<TypeI, AssemblerError> {
    expect_args_count(ctx, 2)?;

    let rt = match ctx.mnemonic {
        "bgez" => 0x01,
        "bgezal" => 0x11,
        "bgtz" => 0x00,
        "blez" => 0x00,
        "bltz" => 0x00,
        "bltzal" => 0x10,
        _ => unreachable!(),
    };

    Ok(TypeI {
        rs: ctx.args[0].as_register()?,
        rt: RegisterName::new(rt),
        imm: ctx.resolve_branch(&ctx.args[1])? as u16,
    })
}

fn try_parse_ins_jump(ctx: &mut LineContext) -> Result<TypeJ, AssemblerError> {
    expect_args_count(ctx, 1)?;

    Ok(TypeJ {
        target: ctx.resolve_jump(&ctx.args[0])?,
    })
}

fn try_parse_ins_jump_reg_linked(ctx: &mut LineContext) -> Result<TypeR, AssemblerError> {
    let rd;
    let rs;

    if ctx.args.len() == 1 {
        rd = RegisterName::new(31);
        rs = ctx.args[0].as_register()?;
    } else if ctx.args.len() == 2 {
        rd = ctx.args[0].as_register()?;
        rs = ctx.args[1].as_register()?;
    } else {
        return InvalidNumberOfOperandsSnafu { line: ctx.line }.fail();
    }

    Ok(TypeR {
        rs,
        rt: RegisterName::new(0),
        rd,
        shamt: 0,
    })
}

fn try_parse_ins_jump_reg(ctx: &mut LineContext) -> Result<TypeR, AssemblerError> {
    expect_args_count(ctx, 1)?;

    Ok(TypeR {
        rs: ctx.args[0].as_register()?,
        rt: RegisterName::new(0),
        rd: RegisterName::new(0),
        shamt: 0,
    })
}

fn try_parse_ins_syscall(ctx: &mut LineContext) -> Result<TypeR, AssemblerError> {
    expect_args_count(ctx, 0)?;

    // All zero except funct
    Ok(Default::default())
}

fn try_parse_ins(ctx: &mut LineContext) -> Result<Instruction, AssemblerError> {
    use Instruction::*;

    Ok(match ctx.mnemonic {
        "nop" => sll(Default::default()),

        "add" => add(try_parse_ins_3arg(ctx)?),
        "addu" => addu(try_parse_ins_3arg(ctx)?),
        "and" => and(try_parse_ins_3arg(ctx)?),
        "nor" => nor(try_parse_ins_3arg(ctx)?),
        "or" => or(try_parse_ins_3arg(ctx)?),
        "slt" => slt(try_parse_ins_3arg(ctx)?),
        "sltu" => sltu(try_parse_ins_3arg(ctx)?),
        "sub" => sub(try_parse_ins_3arg(ctx)?),
        "subu" => subu(try_parse_ins_3arg(ctx)?),
        "xor" => xor(try_parse_ins_3arg(ctx)?),

        "sll" => sll(try_parse_ins_shift_imm(ctx)?),
        "sllv" => sllv(try_parse_ins_shift_reg(ctx)?),
        "sra" => sra(try_parse_ins_shift_imm(ctx)?),
        "srav" => srav(try_parse_ins_shift_reg(ctx)?),
        "srl" => srl(try_parse_ins_shift_imm(ctx)?),
        "srlv" => srlv(try_parse_ins_shift_reg(ctx)?),

        "addi" => addi(try_parse_ins_imm(ctx, true)?),
        "addiu" => addiu(try_parse_ins_imm(ctx, true)?),
        "andi" => andi(try_parse_ins_imm(ctx, false)?),
        "lui" => lui(try_parse_ins_lui(ctx)?),
        "ori" => ori(try_parse_ins_imm(ctx, false)?),
        "slti" => slti(try_parse_ins_imm(ctx, true)?),
        "sltiu" => sltiu(try_parse_ins_imm(ctx, false)?),
        "xori" => xori(try_parse_ins_imm(ctx, false)?),

        "beq" => beq(try_parse_ins_branch(ctx)?),
        "bgez" => bgez(try_parse_ins_branch_complex(ctx)?),
        "bgezal" => bgezal(try_parse_ins_branch_complex(ctx)?),
        "bgtz" => bgtz(try_parse_ins_branch_complex(ctx)?),
        "blez" => blez(try_parse_ins_branch_complex(ctx)?),
        "bltz" => bltz(try_parse_ins_branch_complex(ctx)?),
        "bltzal" => bltzal(try_parse_ins_branch_complex(ctx)?),
        "bne" => bne(try_parse_ins_branch(ctx)?),

        "lb" => lb(try_parse_ins_memory(ctx)?),
        "lbu" => lbu(try_parse_ins_memory(ctx)?),
        "lh" => lh(try_parse_ins_memory(ctx)?),
        "lhu" => lhu(try_parse_ins_memory(ctx)?),
        "lw" => lw(try_parse_ins_memory(ctx)?),
        "sb" => sb(try_parse_ins_memory(ctx)?),
        "sh" => sh(try_parse_ins_memory(ctx)?),
        "sw" => sw(try_parse_ins_memory(ctx)?),

        "j" => j(try_parse_ins_jump(ctx)?),
        "jal" => jal(try_parse_ins_jump(ctx)?),
        "jalr" => jalr(try_parse_ins_jump_reg_linked(ctx)?),
        "jr" => jr(try_parse_ins_jump_reg(ctx)?),
        "syscall" => syscall(try_parse_ins_syscall(ctx)?),

        _ => return UnknownInstructionSnafu { ins: ctx.mnemonic }.fail(),
    })
}

fn parse(
    endian: EndianMode,
    asm: &str,
    labels: &Option<HashMap<String, u32>>,
) -> Result<Vec<Segment>, AssemblerError> {
    let mut segs = vec![];
    let mut curr_seg: Option<Segment> = None;
    let mut global_labels = HashSet::new();
    let mut is_text_seg = false;

    const TEXT_SEGMENT: RangeInclusive<u32> = 0x00400000..=0x0fffffff;
    const DATA_SEGMENT: RangeInclusive<u32> = 0x10000000..=0x7fffffff;

    let mut next_data_addr = 0x10000000;
    let mut next_text_addr = 0x00400024;

    for line_raw in asm.lines() {
        let line = line_raw.trim().to_ascii_lowercase();

        let line = if let Some(comment_pos) = line.find('#') {
            &line[..comment_pos]
        } else {
            &line
        };

        let mut tokens = RE_SEPARATOR.splitn(line, 2);

        let first_token = match tokens.next() {
            Some(x) => x,
            None => continue,
        };

        if first_token.is_empty() {
            continue;
        }

        let args_raw = tokens.next().unwrap_or("");
        let tokens = tokenize(RE_SEPARATOR.split(args_raw));

        if first_token == ".text" || first_token == ".data" {
            if let Some(x) = curr_seg {
                if is_text_seg {
                    next_text_addr = x.next_address();
                } else {
                    next_data_addr = x.next_address();
                }

                segs.push(x);
            }

            let base_addr = if tokens.is_empty() {
                if first_token == ".text" {
                    next_text_addr
                } else {
                    next_data_addr
                }
            } else if tokens.len() == 1 {
                let addr = tokens[0].as_number()?;
                addr.try_into()
                    .map_err(|_| BaseAddressTooLargeSnafu { addr: addr as u64 }.build())?
            } else {
                return InvalidNumberOfOperandsSnafu { line }.fail();
            };

            let seg_type = if first_token == ".text" {
                is_text_seg = true;
                Some(TEXT_SEGMENT)
            } else if first_token == ".data" {
                is_text_seg = false;
                Some(DATA_SEGMENT)
            } else {
                None
            };

            if let Some(x) = seg_type {
                if !x.contains(&base_addr) {
                    return Err(BaseAddressOutOfRangeSnafu {
                        addr: base_addr,
                        range: x,
                    }
                    .build());
                }
            }

            curr_seg = Some(Segment::new(base_addr, endian));
        } else if first_token == ".globl" {
            if curr_seg.is_none() {
                SegmentRequiredSnafu { line }.fail()?;
            }

            for token in &tokens {
                global_labels.insert(token.as_text().to_owned());
            }
        } else if let Some(keyword) = first_token.strip_prefix('.') {
            let seg = curr_seg
                .as_mut()
                .ok_or_else(|| SegmentRequiredSnafu { line }.build())?;

            match keyword {
                "word" => {
                    for token in &tokens {
                        seg.append_u32(token.as_number()? as _);
                    }
                }
                "byte" => {
                    for token in &tokens {
                        seg.append_u8(token.as_number()? as _);
                    }
                }
                "ascii" => {
                    line_raw
                        .trim()
                        .strip_prefix(".ascii")
                        .and_then(|x| x.trim().strip_prefix('"'))
                        .and_then(|x| x.strip_suffix('"'))
                        .map(|x| seg.append_bytes(x.as_bytes()))
                        .ok_or_else(|| InvalidNumberOfOperandsSnafu { line: line_raw }.build())?;
                }
                "asciiz" => {
                    line_raw
                        .trim()
                        .strip_prefix(".asciiz")
                        .and_then(|x| x.trim().strip_prefix('"'))
                        .and_then(|x| x.strip_suffix('"'))
                        .map(|x| {
                            seg.append_bytes(x.as_bytes());
                            seg.append_u8(0);
                        })
                        .ok_or_else(|| InvalidNumberOfOperandsSnafu { line: line_raw }.build())?;
                }
                "float" => {
                    for token in &tokens {
                        let data: f32 = token.as_text().parse().map_err(|_| {
                            InvalidTokenSnafu {
                                token: token.as_text(),
                            }
                            .build()
                        })?;
                        let conv = data.to_bits();
                        seg.append_u32(conv);
                    }
                }
                "align" => {
                    if tokens.len() != 1 {
                        return InvalidNumberOfOperandsSnafu {
                            line: line_raw.trim(),
                        }
                        .fail();
                    }

                    seg.zero_align(tokens[0].as_number()? as usize);
                }
                _ => {
                    if let Some(label) = first_token.strip_suffix(':') {
                        seg.append_label(label);
                    } else {
                        return InvalidTokenSnafu { token: first_token }.fail();
                    }
                }
            }
        } else if let Some(label) = first_token.strip_suffix(':') {
            let seg = curr_seg
                .as_mut()
                .ok_or_else(|| SegmentRequiredSnafu { line }.build())?;

            seg.append_label(label);
        } else {
            let seg = curr_seg
                .as_mut()
                .ok_or_else(|| SegmentRequiredSnafu { line }.build())?;

            let mut start_idx = 0;
            for token in &tokens {
                if let Token::LabelDef { text } = token {
                    start_idx += 1;
                    let label = text
                        .strip_suffix(':')
                        .expect("LabelDef should end with colon");
                    seg.append_label(label);
                } else {
                    break;
                }
            }

            let mut ctx = LineContext {
                line,
                mnemonic: first_token,
                args: &tokens[start_idx..],
                args_raw,
                pc: seg.next_address(),
                labels,
            };

            let ins = try_parse_ins(&mut ctx)?;

            seg.append_u32(ins.encode());
        }
    }

    if let Some(x) = curr_seg {
        segs.push(x);
    }

    Ok(segs)
}

pub fn assemble(endian: EndianMode, asm: &str) -> Result<Vec<Segment>, AssemblerError> {
    // assemble
    let segments = parse(endian, asm, &None)?;

    // collect labels
    let mut labels = HashMap::new();
    for seg in &segments {
        for (k, v) in seg.labels() {
            labels.insert(k.clone(), seg.base_addr + v);
        }
    }

    // reassemble with label
    drop(segments);
    let segments = parse(endian, asm, &Some(labels))?;

    // check overlap
    for a in &segments {
        for b in &segments {
            if std::ptr::eq(a, b) {
                continue;
            }

            if a.overlaps_with(b) {
                return SegmentOverlapSnafu {}.fail();
            }
        }
    }

    Ok(segments)
}

#[cfg(test)]
mod test {
    use super::*;
    use byteorder::{BigEndian, LittleEndian, NativeEndian, ReadBytesExt};
    use std::io::Cursor;

    lazy_static! {
        static ref NE: EndianMode = EndianMode::native();
    }

    #[test]
    fn assemble_empty() {
        assert_eq!(assemble(*NE, "").unwrap().is_empty(), true);
        assert_eq!(assemble(*NE, "\n\n\n\n\n\n").unwrap().is_empty(), true);
    }

    #[test]
    fn assemble_empty_seg() {
        assert_eq!(
            assemble(*NE, ".text\n.data\n.text\n.data").unwrap().len(),
            4
        );
        assert_eq!(
            assemble(*NE, ".data\n.text\n.text\n.text").unwrap().len(),
            4
        );
    }

    #[test]
    fn assemble_arith() {
        let code = ".text\nadd $0, $4, $12\nsub $2, $s0, $zero";
        let segs = assemble(*NE, code).unwrap();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].base_addr, 0x00400024);
        assert_eq!(segs[0].data.len(), 8);
        assert!(segs[0].labels().is_empty());

        let mut data = Cursor::new(&segs[0].data);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x008c0020);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x02001022);
    }

    #[test]
    fn assemble_data() {
        let code = ".data\n.word 123, 0x123, 0o123\n.word 0xffffffff";
        let segs = assemble(*NE, code).unwrap();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].base_addr, 0x10000000);
        assert_eq!(segs[0].data.len(), 16);
        assert!(segs[0].labels().is_empty());

        let mut data = Cursor::new(&segs[0].data);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 123);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x123);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0o123);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0xffffffff);
    }

    #[test]
    fn assemble_memory() {
        let code = ".text\nlw $3, 1234($5)\nsw $s1, -12($gp)\nlw $7, 0x7fff($4)";
        let segs = assemble(*NE, code).unwrap();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].base_addr, 0x00400024);
        assert_eq!(segs[0].data.len(), 12);
        assert!(segs[0].labels().is_empty());

        let mut data = Cursor::new(&segs[0].data);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x8ca304d2);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0xaf91fff4);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x8c877fff);
    }

    #[test]
    fn assemble_memory_fail() {
        let code = ".text\nlw $7, 0x8000($4)";
        let result = assemble(*NE, code);
        assert!(result.is_err());
        if let AssemblerError::ImmediateTooLarge { .. } = result.unwrap_err() {
            // ok
        } else {
            panic!("expected OffsetTooLarge error");
        }
    }

    #[test]
    fn assemble_branch_raw() {
        let code = ".text\nadd $0, $0, $0\nbeq $4, $zero, 92";
        let segs = assemble(*NE, code).unwrap();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].base_addr, 0x00400024);
        assert_eq!(segs[0].data.len(), 8);
        assert!(segs[0].labels().is_empty());

        let mut data = Cursor::new(&segs[0].data);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x00000020);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x10800017);
    }

    #[test]
    fn assemble_branch_label() {
        // Contains no branch delay slot!
        let code = ".text\nbeq $0, $0, fin\n.L1:\nbeq $s0, $28, .L1\nbeq $0, $28, fin\nfin:\nbeq $10, $11, .L1";
        let segs = assemble(*NE, code).unwrap();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].base_addr, 0x00400024);
        assert_eq!(segs[0].data.len(), 16);
        assert_eq!(segs[0].labels().len(), 2);

        let mut data = Cursor::new(&segs[0].data);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x10000002);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x121cffff);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x101c0000);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x114bfffd);
    }

    #[test]
    fn assemble_large_branch() {
        // Contains no branch delay slot!
        let code = ".text\nbeq $0, $0, -131072\nbeq $0, $0, 131068";
        let segs = assemble(*NE, code).unwrap();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].base_addr, 0x00400024);
        assert_eq!(segs[0].data.len(), 8);
        assert!(segs[0].labels().is_empty());

        let mut data = Cursor::new(&segs[0].data);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x10008000);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x10007fff);
    }

    #[test]
    fn assemble_jump() {
        // Contains no branch delay slot!
        let code = ".text 0x00400024\nbeq $0, $0, fin\n.L1:\nadd $s1, $0, $0\nfin:\nj .L1";
        let segs = assemble(*NE, code).unwrap();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].base_addr, 0x00400024);
        assert_eq!(segs[0].data.len(), 12);
        assert_eq!(segs[0].labels().len(), 2);

        let mut data = Cursor::new(&segs[0].data);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x10000001);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x00008820);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x0810000a);
    }

    #[test]
    fn assemble_imm() {
        let code = r"
        .text
        addi $0, $0, 32767
        addiu $0, $0, -32768
        andi $s1, $s2, 0xffff
        lui $v1, 0o177777
        ori $6, $10, 1234
        slti $0, $0, -1234
        sltiu $0, $0, 0
        xori $0, $0, 10101";

        let segs = assemble(*NE, code).unwrap();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].base_addr, 0x00400024);
        assert_eq!(segs[0].data.len(), 32);
        assert!(segs[0].labels().is_empty());

        let mut data = Cursor::new(&segs[0].data);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x20007fff);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x24008000);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x3251ffff);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x3c03ffff);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x354604d2);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x2800fb2e);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x2c000000);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x38002775);
    }

    #[test]
    fn assemble_imm_fails() {
        let err1 = assemble(*NE, ".text\naddi $0, $0, 32768").expect_err("must result in error");

        if let AssemblerError::ImmediateTooLarge { .. } = &err1 {
            // ok
        } else {
            panic!("expected ImmediateTooLarge, got {:?}", err1);
        }

        let err2 = assemble(*NE, ".text\naddi $0, $0, -32769").expect_err("must result in error");

        if let AssemblerError::ImmediateTooLarge { .. } = &err2 {
            // ok
        } else {
            panic!("expected ImmediateTooLarge, got {:?}", err2);
        }
    }

    #[test]
    fn assemble_shifts() {
        let code = r"
        .text
        sll $v0, $v1, 0
        sllv $s0, $s1, $16
        sra $0, $16, 31
        srav $0, $16, $s2
        srl $v1, $s0, 0x3
        srlv $s0, $s1, $0";

        let segs = assemble(*NE, code).unwrap();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].base_addr, 0x00400024);
        assert_eq!(segs[0].data.len(), 24);
        assert!(segs[0].labels().is_empty());

        let mut data = Cursor::new(&segs[0].data);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x00031000);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x02118004);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x001007c3);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x02500007);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x001018c2);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x00118006);
    }

    #[test]
    fn assemble_branches() {
        let code = r"
        .text 0x00400024
        start:
        jal start
        jalr $a0
        jalr $30, $20
        syscall
        jr $s0
        bgez $20, start
        bgezal $v0, start
        bltz $v1, start
        bltzal $s0, start
        bgtz $sp, start
        blez $sp, start";

        let segs = assemble(*NE, code).unwrap();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].base_addr, 0x00400024);
        assert_eq!(segs[0].data.len(), 44);
        assert_eq!(segs[0].labels().len(), 1);
        assert!(segs[0].labels().contains_key("start"));

        let mut data = Cursor::new(&segs[0].data);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x0c100009);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x0080f809);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x0280f009);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x0000000c);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x02000008);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x0681fffa);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x0451fff9);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x0460fff8);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x0610fff7);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x1fa0fff6);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x1ba0fff5);
    }

    #[test]
    fn extra_data_directives() {
        let code = r#"
        .data 0x10008024
        .word 1 2 3
        .word -1 -2 -3
        .ascii "asdf"
        .asciiz "abc"
        .byte 1 2 3 4 5
        .byte -1 -2
        .align 4
        .float 1.2"#;

        // little endian test
        let segs = assemble(EndianMode::Little, code).unwrap();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].base_addr, 0x10008024);
        assert_eq!(segs[0].data.len(), 44);

        let mut data = Cursor::new(&segs[0].data);
        assert_eq!(data.read_u32::<LittleEndian>().unwrap(), 1);
        assert_eq!(data.read_u32::<LittleEndian>().unwrap(), 2);
        assert_eq!(data.read_u32::<LittleEndian>().unwrap(), 3);
        assert_eq!(data.read_u32::<LittleEndian>().unwrap(), -1_i32 as u32);
        assert_eq!(data.read_u32::<LittleEndian>().unwrap(), -2_i32 as u32);
        assert_eq!(data.read_u32::<LittleEndian>().unwrap(), -3_i32 as u32);
        assert_eq!(data.read_u32::<LittleEndian>().unwrap(), 0x66647361);
        assert_eq!(data.read_u32::<LittleEndian>().unwrap(), 0x00636261);
        assert_eq!(data.read_u32::<LittleEndian>().unwrap(), 0x04030201);
        assert_eq!(data.read_u32::<LittleEndian>().unwrap(), 0x00FEFF05);
        assert_eq!(data.read_u32::<LittleEndian>().unwrap(), 0x3F99999A);

        drop(data);
        drop(segs);

        // big endian test
        let segs = assemble(EndianMode::Big, code).unwrap();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].base_addr, 0x10008024);
        assert_eq!(segs[0].data.len(), 44);

        let mut data = Cursor::new(&segs[0].data);
        assert_eq!(data.read_u32::<BigEndian>().unwrap(), 1);
        assert_eq!(data.read_u32::<BigEndian>().unwrap(), 2);
        assert_eq!(data.read_u32::<BigEndian>().unwrap(), 3);
        assert_eq!(data.read_u32::<BigEndian>().unwrap(), -1_i32 as u32);
        assert_eq!(data.read_u32::<BigEndian>().unwrap(), -2_i32 as u32);
        assert_eq!(data.read_u32::<BigEndian>().unwrap(), -3_i32 as u32);
        assert_eq!(data.read_u32::<BigEndian>().unwrap(), 0x61736466);
        assert_eq!(data.read_u32::<BigEndian>().unwrap(), 0x61626300);
        assert_eq!(data.read_u32::<BigEndian>().unwrap(), 0x01020304);
        assert_eq!(data.read_u32::<BigEndian>().unwrap(), 0x05FFFE00);
        assert_eq!(data.read_u32::<BigEndian>().unwrap(), 0x3F99999A);
    }
}
