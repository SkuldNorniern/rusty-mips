use super::error::*;
use crate::component::{Instruction, RegisterName, TypeI, TypeJ, TypeR};
use crate::memory::{EndianMode, Segment};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::mem::swap;
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

fn try_parse_unsigned(text: &str) -> Result<u64, AssemblerError> {
    let text = text.to_ascii_lowercase();

    if let Some(x) = text.strip_prefix("0x") {
        u64::from_str_radix(x, 16).ok()
    } else if let Some(x) = text.strip_prefix("0o") {
        u64::from_str_radix(x, 8).ok()
    } else if let Some(x) = text.strip_prefix("0b") {
        u64::from_str_radix(x, 2).ok()
    } else {
        u64::from_str(&text).ok()
    }
    .ok_or_else(|| InvalidTokenSnafu { token: text }.build())
}

fn try_parse_signed(text: &str) -> Result<i64, AssemblerError> {
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
    .ok_or_else(|| InvalidTokenSnafu { token: text }.build())
}

fn try_parse_reg(name: &str) -> Result<RegisterName, AssemblerError> {
    name.strip_prefix('$')
        .and_then(RegisterName::try_from_name)
        .ok_or_else(|| InvalidRegisterNameSnafu { reg: name }.build())
}

fn bail_trailing_token<'a>(mut iter: impl Iterator<Item = &'a str>) -> Result<(), AssemblerError> {
    match iter.next() {
        Some(x) => Err(TrailingTokenSnafu {
            token: x.to_owned(),
        }
        .build()),
        None => Ok(()),
    }
}

fn try_parse_ins_3arg(args: &str, line: &str, shift_order: bool) -> Result<TypeR, AssemblerError> {
    let mut args = args.split(',');

    let rd = args
        .next()
        .ok_or_else(|| InvalidNumberOfOperandsSnafu { line }.build())?;
    let mut rs = args
        .next()
        .ok_or_else(|| InvalidNumberOfOperandsSnafu { line }.build())?;
    let mut rt = args
        .next()
        .ok_or_else(|| InvalidNumberOfOperandsSnafu { line }.build())?;

    bail_trailing_token(args)?;

    if shift_order {
        // Shift instructions use $d, $t, $s order,
        // where normal ones use $d, $s, $t order.
        swap(&mut rs, &mut rt);
    }

    Ok(TypeR {
        rs: try_parse_reg(rs.trim())?,
        rt: try_parse_reg(rt.trim())?,
        rd: try_parse_reg(rd.trim())?,
        shamt: 0,
    })
}

fn try_parse_ins_imm(args: &str, line: &str, sign_ext: bool) -> Result<TypeI, AssemblerError> {
    let mut args = args.split(',');

    let rt = args
        .next()
        .ok_or_else(|| InvalidNumberOfOperandsSnafu { line }.build())?;
    let rs = args
        .next()
        .ok_or_else(|| InvalidNumberOfOperandsSnafu { line }.build())?;
    let imm = args
        .next()
        .ok_or_else(|| InvalidNumberOfOperandsSnafu { line }.build())
        .and_then(|x| try_parse_signed(x.trim()))?;

    bail_trailing_token(args)?;

    let interpreted = if sign_ext {
        imm as i16 as i64
    } else {
        imm as u16 as u64 as i64
    };
    if interpreted != imm {
        return Err(ImmediateTooLargeSnafu { imm }.build());
    }

    Ok(TypeI {
        rs: try_parse_reg(rs.trim())?,
        rt: try_parse_reg(rt.trim())?,
        imm: imm as u16,
    })
}

fn try_parse_ins_lui(args: &str, line: &str, sign_ext: bool) -> Result<TypeI, AssemblerError> {
    let mut args = args.split(',');

    let rt = args
        .next()
        .ok_or_else(|| InvalidNumberOfOperandsSnafu { line }.build())?;
    let imm = args
        .next()
        .ok_or_else(|| InvalidNumberOfOperandsSnafu { line }.build())
        .and_then(|x| try_parse_signed(x.trim()))?;

    bail_trailing_token(args)?;

    let interpreted = if sign_ext {
        imm as i16 as i64
    } else {
        imm as u16 as u64 as i64
    };
    if interpreted != imm {
        return Err(ImmediateTooLargeSnafu { imm }.build());
    }

    Ok(TypeI {
        rs: RegisterName::new(0),
        rt: try_parse_reg(rt.trim())?,
        imm: imm as u16,
    })
}

fn try_parse_ins_shift(args: &str, line: &str) -> Result<TypeR, AssemblerError> {
    let mut args = args.split(',');

    let rd = args
        .next()
        .ok_or_else(|| InvalidNumberOfOperandsSnafu { line }.build())?;
    let rt = args
        .next()
        .ok_or_else(|| InvalidNumberOfOperandsSnafu { line }.build())?;
    let imm = args
        .next()
        .ok_or_else(|| InvalidNumberOfOperandsSnafu { line }.build())
        .and_then(|x| try_parse_unsigned(x.trim()))?;

    bail_trailing_token(args)?;

    if 32 <= imm {
        return ImmediateTooLargeSnafu { imm: imm as i64 }.fail();
    }

    Ok(TypeR {
        rs: RegisterName::new(0),
        rt: try_parse_reg(rt.trim())?,
        rd: try_parse_reg(rd.trim())?,
        shamt: imm as u8,
    })
}

fn try_parse_ins_memory(args: &str, line: &str) -> Result<TypeI, AssemblerError> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(\$.+)\s*,\s*([^ ]+?)\((\$.+)\)").unwrap();
    }

    let caps = match RE.captures(args) {
        Some(x) => x,
        None => InvalidNumberOfOperandsSnafu { line }.fail()?,
    };

    let imm = try_parse_signed(&caps[2])?;
    let rs = try_parse_reg(&caps[3])?;
    let rt = try_parse_reg(&caps[1])?;

    if TryInto::<i16>::try_into(imm).is_err() {
        OffsetTooLargeSnafu { offset: imm }.fail()?;
    }

    Ok(TypeI {
        rs,
        rt,
        imm: imm as u16,
    })
}

fn try_parse_ins_branch(
    args: &str,
    line: &str,
    pc: u32,
    labels: &Option<HashMap<String, u32>>,
) -> Result<TypeI, AssemblerError> {
    let mut args = args.split(',');

    let rs = args
        .next()
        .ok_or_else(|| InvalidNumberOfOperandsSnafu { line }.build())
        .map(|x| x.trim())
        .and_then(try_parse_reg)?;
    let rt = args
        .next()
        .ok_or_else(|| InvalidNumberOfOperandsSnafu { line }.build())
        .map(|x| x.trim())
        .and_then(try_parse_reg)?;
    let label = args
        .next()
        .ok_or_else(|| InvalidNumberOfOperandsSnafu { line }.build())
        .map(|x| x.trim())?;

    bail_trailing_token(args)?;

    let offset = if let Ok(x) = try_parse_signed(label) {
        // parse as offset
        x
    } else if let Some(map) = labels {
        // parse as label
        map.get(label)
            .map(|x| *x as i64 - pc as i64 - 4)
            .ok_or_else(|| LabelNotFoundSnafu { label }.build())?
    } else {
        // parse as label, but it's first pass; treat as 0
        0
    };

    if offset % 4 != 0 {
        BranchOffsetUnalignedSnafu { offset }.fail()?;
    }

    let offset: i16 = (offset / 4)
        .try_into()
        .map_err(|_| OffsetTooLargeSnafu { offset }.build())?;

    Ok(TypeI {
        rs,
        rt,
        imm: offset as u16,
    })
}

fn try_parse_ins_branch_extra(
    args: &str,
    line: &str,
    pc: u32,
    labels: &Option<HashMap<String, u32>>,
    mnemonic: &str,
) -> Result<TypeI, AssemblerError> {
    let mut args = args.split(',');

    let rs = args
        .next()
        .ok_or_else(|| InvalidNumberOfOperandsSnafu { line }.build())
        .map(|x| x.trim())
        .and_then(try_parse_reg)?;
    let label = args
        .next()
        .ok_or_else(|| InvalidNumberOfOperandsSnafu { line }.build())
        .map(|x| x.trim())?;

    bail_trailing_token(args)?;

    let rt = match mnemonic {
        "bgez" => 0x01,
        "bgezal" => 0x11,
        "bgtz" => 0x00,
        "blez" => 0x00,
        "bltz" => 0x00,
        "bltzal" => 0x10,
        _ => unreachable!(),
    };

    let offset = if let Ok(x) = try_parse_signed(label) {
        // parse as offset
        x
    } else if let Some(map) = labels {
        // parse as label
        map.get(label)
            .map(|x| *x as i64 - pc as i64 - 4)
            .ok_or_else(|| LabelNotFoundSnafu { label }.build())?
    } else {
        // parse as label, but it's first pass; treat as 0
        0
    };

    if offset % 4 != 0 {
        BranchOffsetUnalignedSnafu { offset }.fail()?;
    }

    let offset: i16 = (offset / 4)
        .try_into()
        .map_err(|_| OffsetTooLargeSnafu { offset }.build())?;

    Ok(TypeI {
        rs,
        rt: RegisterName::new(rt),
        imm: offset as u16,
    })
}

fn try_parse_ins_jump(
    args: &str,
    line: &str,
    pc: u32,
    labels: &Option<HashMap<String, u32>>,
) -> Result<TypeJ, AssemblerError> {
    let mut args = args.split(',');

    let label = args
        .next()
        .ok_or_else(|| InvalidNumberOfOperandsSnafu { line }.build())
        .map(|x| x.trim())?;

    bail_trailing_token(args)?;

    let target = if let Ok(x) = try_parse_unsigned(label) {
        // parse as target
        x as u32
    } else if let Some(map) = labels {
        // parse as label
        map.get(label)
            .ok_or_else(|| LabelNotFoundSnafu { label }.build())
            .map(|x| *x)?
    } else {
        // parse as label, but it's first pass; treat as 0
        0
    };

    if target % 4 != 0 {
        JumpTargetUnalignedSnafu { target }.fail()?;
    }
    if target & 0xF000_0000 != (pc + 4) & 0xF000_0000 {
        JumpTooFarSnafu { target, pc }.fail()?;
    }

    Ok(TypeJ {
        target: (target / 4) & 0x03FF_FFFF,
    })
}

fn try_parse_ins_jump_reg(
    args: &str,
    line: &str,
    accept_link: bool,
) -> Result<TypeR, AssemblerError> {
    let mut args = args.split(',');

    let arg0 = args
        .next()
        .ok_or_else(|| InvalidNumberOfOperandsSnafu { line }.build())
        .map(|x| x.trim())?;

    let arg1 = args.next().map(|x| x.trim());

    bail_trailing_token(args)?;

    let rs;
    let rd;

    if accept_link {
        if arg1.is_some() {
            rs = try_parse_reg(arg1.unwrap())?;
            rd = try_parse_reg(arg0)?;
        } else {
            rs = try_parse_reg(arg0)?;
            rd = RegisterName::new(31);
        }
    } else {
        if arg1.is_some() {
            return InvalidNumberOfOperandsSnafu { line }.fail();
        } else {
            rs = try_parse_reg(arg0)?;
            rd = RegisterName::new(0);
        }
    }

    Ok(TypeR {
        rs,
        rt: RegisterName::new(0),
        rd,
        shamt: 0,
    })
}

fn try_parse_syscall(args: &str, line: &str) -> Result<TypeR, AssemblerError> {
    if !args.trim_start().is_empty() {
        return InvalidNumberOfOperandsSnafu { line }.fail();
    }

    // All zero except funct
    Ok(Default::default())
}

fn try_parse_ins(
    line: &str,
    mnemonic: &str,
    pc: u32,
    labels: &Option<HashMap<String, u32>>,
) -> Result<Instruction, AssemblerError> {
    use Instruction::*;

    let args = line
        .trim()
        .strip_prefix(mnemonic)
        .expect("line should start with mnemonic")
        .trim_start();

    Ok(match mnemonic {
        "add" => add(try_parse_ins_3arg(args, line, false)?),
        "addu" => addu(try_parse_ins_3arg(args, line, false)?),
        "and" => and(try_parse_ins_3arg(args, line, false)?),
        "nor" => nor(try_parse_ins_3arg(args, line, false)?),
        "or" => or(try_parse_ins_3arg(args, line, false)?),
        "slt" => slt(try_parse_ins_3arg(args, line, false)?),
        "sltu" => sltu(try_parse_ins_3arg(args, line, false)?),
        "sub" => sub(try_parse_ins_3arg(args, line, false)?),
        "subu" => subu(try_parse_ins_3arg(args, line, false)?),
        "xor" => xor(try_parse_ins_3arg(args, line, false)?),

        "sll" => sll(try_parse_ins_shift(args, line)?),
        "sllv" => sllv(try_parse_ins_3arg(args, line, true)?),
        "sra" => sra(try_parse_ins_shift(args, line)?),
        "srav" => srav(try_parse_ins_3arg(args, line, true)?),
        "srl" => srl(try_parse_ins_shift(args, line)?),
        "srlv" => srlv(try_parse_ins_3arg(args, line, true)?),

        "addi" => addi(try_parse_ins_imm(args, line, true)?),
        "addiu" => addiu(try_parse_ins_imm(args, line, true)?),
        "andi" => andi(try_parse_ins_imm(args, line, false)?),
        "lui" => lui(try_parse_ins_lui(args, line, false)?),
        "ori" => ori(try_parse_ins_imm(args, line, false)?),
        "slti" => slti(try_parse_ins_imm(args, line, true)?),
        "sltiu" => sltiu(try_parse_ins_imm(args, line, false)?),
        "xori" => xori(try_parse_ins_imm(args, line, false)?),

        "beq" => beq(try_parse_ins_branch(args, line, pc, labels)?),
        "bgez" => bgez(try_parse_ins_branch_extra(
            args, line, pc, labels, mnemonic,
        )?),
        "bgezal" => bgezal(try_parse_ins_branch_extra(
            args, line, pc, labels, mnemonic,
        )?),
        "bgtz" => bgtz(try_parse_ins_branch_extra(
            args, line, pc, labels, mnemonic,
        )?),
        "blez" => blez(try_parse_ins_branch_extra(
            args, line, pc, labels, mnemonic,
        )?),
        "bltz" => bltz(try_parse_ins_branch_extra(
            args, line, pc, labels, mnemonic,
        )?),
        "bltzal" => bltzal(try_parse_ins_branch_extra(
            args, line, pc, labels, mnemonic,
        )?),
        "bne" => bne(try_parse_ins_branch(args, line, pc, labels)?),

        "lb" => lb(try_parse_ins_memory(args, line)?),
        "lbu" => lbu(try_parse_ins_memory(args, line)?),
        "lh" => lh(try_parse_ins_memory(args, line)?),
        "lhu" => lhu(try_parse_ins_memory(args, line)?),
        "lw" => lw(try_parse_ins_memory(args, line)?),
        "sb" => sb(try_parse_ins_memory(args, line)?),
        "sh" => sh(try_parse_ins_memory(args, line)?),
        "sw" => sw(try_parse_ins_memory(args, line)?),

        "j" => j(try_parse_ins_jump(args, line, pc, labels)?),
        "jal" => jal(try_parse_ins_jump(args, line, pc, labels)?),
        "jalr" => jalr(try_parse_ins_jump_reg(args, line, true)?),
        "jr" => jr(try_parse_ins_jump_reg(args, line, false)?),
        "syscall" => syscall(try_parse_syscall(args, line)?),

        _ => return UnknownInstructionSnafu { ins: mnemonic }.fail(),
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
    let mut next_text_addr = 0x00400000;

    for line in asm.lines() {
        let mut line = line.trim();

        if let Some(comment_pos) = line.find('#') {
            line = &line[..comment_pos];
        }

        if line.is_empty() {
            continue;
        }

        let mut tokens = line.split_whitespace();

        // unwrap safety: trimmed and non-empty (thus contains at least one non-whitespace character)
        let first_token = tokens.next().unwrap();

        if first_token == ".text" || first_token == ".data" {
            if let Some(x) = curr_seg {
                if is_text_seg {
                    next_text_addr = x.next_address();
                } else {
                    next_data_addr = x.next_address();
                }

                segs.push(x);
            }

            let base_addr = match tokens.next().and_then(|x| try_parse_unsigned(x).ok()) {
                Some(x) => x as u32,
                None => {
                    if first_token == ".text" {
                        next_text_addr
                    } else {
                        next_data_addr
                    }
                }
            };

            let seg_type;
            if first_token == ".text" {
                is_text_seg = true;
                seg_type = Some(TEXT_SEGMENT);
            } else if first_token == ".data" {
                is_text_seg = false;
                seg_type = Some(DATA_SEGMENT);
            } else {
                seg_type = None;
            }

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
            bail_trailing_token(tokens)?;
        } else if first_token == ".globl" {
            let label = tokens
                .next()
                .ok_or_else(|| RequiredArgNotFoundSnafu {}.build())?;

            if curr_seg.is_none() {
                SegmentRequiredSnafu { line }.fail()?;
            }

            global_labels.insert(label.to_owned());
        } else if first_token == ".word" {
            let seg = curr_seg
                .as_mut()
                .ok_or_else(|| SegmentRequiredSnafu { line }.build())?;

            let values = line
                .strip_prefix(first_token)
                .expect("line should start with first token")
                .split(',')
                .map(|x| try_parse_signed(x.trim()));

            for num in values {
                seg.append_u32(num? as u32);
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
            let pc = seg.next_address();
            let ins = try_parse_ins(line, first_token, pc, labels)?;

            seg.append_u32(ins.encode());
        }
    }

    if let Some(x) = curr_seg {
        segs.push(x);
    }

    Ok(segs)
}

#[allow(unused)]
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
    use byteorder::{NativeEndian, ReadBytesExt};
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
        assert_eq!(segs[0].base_addr, 0x00400000);
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
        assert_eq!(segs[0].base_addr, 0x00400000);
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
        if let AssemblerError::OffsetTooLarge { .. } = result.unwrap_err() {
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
        assert_eq!(segs[0].base_addr, 0x00400000);
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
        assert_eq!(segs[0].base_addr, 0x00400000);
        assert_eq!(segs[0].data.len(), 16);
        assert_eq!(segs[0].labels().len(), 2);

        let mut data = Cursor::new(&segs[0].data);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x10000002);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x121cffff);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x101c0000);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x114bfffd);
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
        assert_eq!(segs[0].base_addr, 0x00400000);
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
        assert_eq!(segs[0].base_addr, 0x00400000);
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
}
