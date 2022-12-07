use super::error::AssemblerError;
use super::instruction::{FormatI, FormatR, Instruction};
use super::segment::Segment;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::ops::RangeInclusive;
use std::str::FromStr;
use lazy_static::lazy_static;
use regex::Regex;
use crate::assembler::instruction::FormatJ;
use crate::component::RegisterName;

fn try_parse_unsigned(text: &str) -> Option<u64> {
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
}

fn try_parse_signed(text: &str) -> Option<i64> {
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

fn try_parse_reg(name: &str) -> Result<RegisterName, AssemblerError> {
    name.strip_prefix('$')
        .and_then(|x| RegisterName::try_from_name(x))
        .ok_or_else(|| AssemblerError::InvalidRegisterName(name.into()))
}

fn bail_trailing_token<'a>(mut iter: impl Iterator<Item = &'a str>) -> Result<(), AssemblerError> {
    match iter.next() {
        Some(x) => Err(AssemblerError::TrailingToken(x.into())),
        None => Ok(()),
    }
}

fn try_parse_ins_3arg(args: &str, line: &str) -> Result<FormatR, AssemblerError> {
    let mut args = args.split(',');

    let rd = args
        .next()
        .ok_or_else(|| AssemblerError::InvalidNumberOfOperands(line.into()))?;
    let rs = args
        .next()
        .ok_or_else(|| AssemblerError::InvalidNumberOfOperands(line.into()))?;
    let rt = args
        .next()
        .ok_or_else(|| AssemblerError::InvalidNumberOfOperands(line.into()))?;

    bail_trailing_token(args)?;

    Ok(FormatR::new(
        try_parse_reg(rd.trim())?,
        try_parse_reg(rs.trim())?,
        try_parse_reg(rt.trim())?,
        0,
    ))
}

fn try_parse_ins_memory(args: &str, line: &str) -> Result<FormatI, AssemblerError> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(\$.+)\s*,\s*([^ ]+?)\((\$.+)\)").unwrap();
    }

    let caps = match RE.captures(args) {
        Some(x) => x,
        None => return Err(AssemblerError::InvalidNumberOfOperands(line.into()))
    };

    let imm = try_parse_signed(&caps[2]).ok_or_else(|| AssemblerError::InvalidToken(caps[2].into()))?;
    let rs = try_parse_reg(&caps[3])?;
    let rt = try_parse_reg(&caps[1])?;

    if TryInto::<i16>::try_into(imm).is_err() {
        return Err(AssemblerError::OffsetTooLarge(imm));
    }

    Ok(FormatI::new(rs, rt, imm as u16))
}

fn try_parse_ins_branch(args: &str, line: &str, pc: u32, labels: &Option<HashMap<String, u32>>) -> Result<FormatI, AssemblerError> {
    let mut args = args.split(',');

    let rs = args
        .next()
        .ok_or_else(|| AssemblerError::InvalidNumberOfOperands(line.into()))
        .map(|x| x.trim())
        .and_then(try_parse_reg)?;
    let rt = args
        .next()
        .ok_or_else(|| AssemblerError::InvalidNumberOfOperands(line.into()))
        .map(|x| x.trim())
        .and_then(try_parse_reg)?;
    let label = args
        .next()
        .ok_or_else(|| AssemblerError::InvalidNumberOfOperands(line.into()))
        .map(|x| x.trim())?;

    bail_trailing_token(args)?;

    let offset = if let Some(x) = try_parse_signed(label) {
        // parse as offset
        x
    } else if let Some(map) = labels {
        // parse as label
        map.get(label)
            .map(|x| *x as i64 - pc as i64 - 4)
            .ok_or_else(|| AssemblerError::LabelNotFound(label.into()))?
    } else {
        // parse as label, but it's first pass; treat as 0
        0
    };

    if offset % 4 != 0 {
        return Err(AssemblerError::BranchOffsetUnaligned(offset));
    }

    let offset: i16 = (offset / 4).try_into().map_err(|_| AssemblerError::OffsetTooLarge(offset))?;

    Ok(FormatI::new(rs, rt, offset as u16))
}

fn try_parse_ins_jump(args: &str, line: &str, pc: u32, labels: &Option<HashMap<String, u32>>) -> Result<FormatJ, AssemblerError> {
    let mut args = args.split(',');

    let label = args
        .next()
        .ok_or_else(|| AssemblerError::InvalidNumberOfOperands(line.into()))
        .map(|x| x.trim())?;

    bail_trailing_token(args)?;

    let target = if let Some(x) = try_parse_unsigned(label) {
        // parse as target
        x as u32
    } else if let Some(map) = labels {
        // parse as label
        map.get(label)
            .ok_or_else(|| AssemblerError::LabelNotFound(label.into()))
            .map(|x| *x)?
    } else {
        // parse as label, but it's first pass; treat as 0
        0
    };

    if target % 4 != 0 {
        return Err(AssemblerError::JumpTargetUnaligned(target));
    }
    if target & 0xF000_0000 != (pc + 4) & 0xF000_0000 {
        return Err(AssemblerError::JumpTooFar { target, pc })
    }

    Ok(FormatJ::new((target / 4) & 0x03FF_FFFF))
}

fn try_parse_ins(
    line: &str,
    mnemonic: &str,
    pc: u32,
    labels: &Option<HashMap<String, u32>>,
) -> Result<Instruction, AssemblerError> {
    let args = line
        .trim()
        .strip_prefix(mnemonic)
        .expect("line should start with mnemonic")
        .trim_start();

    Ok(match mnemonic {
        "and" => Instruction::And(try_parse_ins_3arg(args, line)?),
        "or" => Instruction::Or(try_parse_ins_3arg(args, line)?),
        "add" => Instruction::Add(try_parse_ins_3arg(args, line)?),
        "sub" => Instruction::Sub(try_parse_ins_3arg(args, line)?),
        "slt" => Instruction::Slt(try_parse_ins_3arg(args, line)?),
        "lw" => Instruction::Lw(try_parse_ins_memory(args, line)?),
        "sw" => Instruction::Sw(try_parse_ins_memory(args, line)?),
        "beq" => Instruction::Beq(try_parse_ins_branch(args, line, pc, labels)?),
        "j" => Instruction::J(try_parse_ins_jump(args, line, pc, labels)?),
        _ => return Err(AssemblerError::UnknownInstruction(mnemonic.into())),
    })
}

fn range_overlaps(a: RangeInclusive<u32>, b: RangeInclusive<u32>) -> bool {
    !a.is_empty() && !b.is_empty() && a.start() <= b.end() && b.start() <= a.end()
}

fn parse(asm: &str, labels: &Option<HashMap<String, u32>>) -> Result<Vec<Segment>, AssemblerError> {
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
                    next_text_addr = x.base_addr + x.data.len() as u32;
                } else {
                    next_data_addr = x.base_addr + x.data.len() as u32;
                }

                segs.push(x);
            }

            let base_addr = match tokens.next().and_then(try_parse_unsigned) {
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
                    return Err(AssemblerError::BaseAddressOutOfRange(base_addr, x));
                }
            }

            curr_seg = Some(Segment::new(base_addr));
            bail_trailing_token(tokens)?;
        } else if first_token == ".globl" {
            let label = tokens.next().ok_or(AssemblerError::RequiredArgNotFound)?;

            if curr_seg.is_none() {
                return Err(AssemblerError::SegmentRequired(line.into()));
            }

            global_labels.insert(label.to_owned());
        } else if first_token == ".word" {
            let seg = curr_seg
                .as_mut()
                .ok_or_else(|| AssemblerError::SegmentRequired(line.into()))?;

            let values = line
                .strip_prefix(first_token)
                .expect("line should start with first token")
                .split(',')
                .map(|x| {
                    try_parse_signed(x.trim()).ok_or_else(|| AssemblerError::InvalidToken(x.into()))
                });

            for num in values {
                seg.data.extend_from_slice(&(num? as u32).to_ne_bytes());
            }
        } else if let Some(label) = first_token.strip_suffix(':') {
            let seg = curr_seg
                .as_mut()
                .ok_or_else(|| AssemblerError::SegmentRequired(line.into()))?;

            seg.labels.insert(label.into(), seg.data.len() as u32);
        } else {
            let seg = curr_seg
                .as_mut()
                .ok_or_else(|| AssemblerError::SegmentRequired(line.into()))?;
            let pc = seg.base_addr + seg.data.len() as u32;
            let ins = try_parse_ins(line, first_token, pc, labels)?;

            seg.data.extend_from_slice(&ins.encode().to_ne_bytes());
        }
    }

    if let Some(x) = curr_seg {
        segs.push(x);
    }

    Ok(segs)
}

#[allow(unused)]
pub fn assemble(asm: &str) -> Result<Vec<Segment>, AssemblerError> {
    // assemble
    let segments = parse(asm, &None)?;

    // collect labels
    let mut labels = HashMap::new();
    for seg in &segments {
        for (k, v) in &seg.labels {
            labels.insert(k.clone(), seg.base_addr + v);
        }
    }

    // reassemble with label
    drop(segments);
    let segments = parse(asm, &Some(labels))?;

    // check overlap
    for a in &segments {
        for b in &segments {
            if std::ptr::eq(a, b) {
                continue;
            }

            if range_overlaps(a.address_range(), b.address_range()) {
                return Err(AssemblerError::SegmentOverlap);
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

    #[test]
    fn assemble_empty() {
        assert_eq!(assemble("").unwrap().is_empty(), true);
        assert_eq!(assemble("\n\n\n\n\n\n").unwrap().is_empty(), true);
    }

    #[test]
    fn assemble_empty_seg() {
        assert_eq!(assemble(".text\n.data\n.text\n.data").unwrap().len(), 4);
        assert_eq!(assemble(".data\n.text\n.text\n.text").unwrap().len(), 4);
    }

    #[test]
    fn assemble_arith() {
        let code = ".text\nadd $0, $4, $12\nsub $2, $s0, $zero";
        let segs = assemble(code).unwrap();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].base_addr, 0x00400000);
        assert_eq!(segs[0].data.len(), 8);
        assert!(segs[0].labels.is_empty());

        let mut data = Cursor::new(&segs[0].data);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x008c0020);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x02001022);
    }

    #[test]
    fn assemble_data() {
        let code = ".data\n.word 123, 0x123, 0o123\n.word 0xffffffff";
        let segs = assemble(code).unwrap();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].base_addr, 0x10000000);
        assert_eq!(segs[0].data.len(), 16);
        assert!(segs[0].labels.is_empty());

        let mut data = Cursor::new(&segs[0].data);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 123);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x123);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0o123);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0xffffffff);
    }

    #[test]
    fn assemble_memory() {
        let code = ".text\nlw $3, 1234($5)\nsw $s1, -12($gp)\nlw $7, 0x7fff($4)";
        let segs = assemble(code).unwrap();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].base_addr, 0x00400000);
        assert_eq!(segs[0].data.len(), 12);
        assert!(segs[0].labels.is_empty());

        let mut data = Cursor::new(&segs[0].data);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x8ca304d2);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0xaf91fff4);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x8c877fff);
    }

    #[test]
    fn assemble_memory_fail() {
        let code = ".text\nlw $7, 0x8000($4)";
        let result = assemble(code);
        assert!(result.is_err());
        if let AssemblerError::OffsetTooLarge(_) = result.unwrap_err() {
            // ok
        } else {
            panic!("expected OffsetTooLarge error");
        }
    }

    #[test]
    fn assemble_branch_raw() {
        let code = ".text\nadd $0, $0, $0\nbeq $4, $zero, 92";
        let segs = assemble(code).unwrap();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].base_addr, 0x00400000);
        assert_eq!(segs[0].data.len(), 8);
        assert!(segs[0].labels.is_empty());

        let mut data = Cursor::new(&segs[0].data);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x00000020);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x10800017);
    }

    #[test]
    fn assemble_branch_label() {
        // Contains no branch delay slot!
        let code = ".text\nbeq $0, $0, fin\n.L1:\nbeq $s0, $28, .L1\nbeq $0, $28, fin\nfin:\nbeq $10, $11, .L1";
        let segs = assemble(code).unwrap();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].base_addr, 0x00400000);
        assert_eq!(segs[0].data.len(), 16);
        assert_eq!(segs[0].labels.len(), 2);

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
        let segs = assemble(code).unwrap();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].base_addr, 0x00400024);
        assert_eq!(segs[0].data.len(), 12);
        assert_eq!(segs[0].labels.len(), 2);

        let mut data = Cursor::new(&segs[0].data);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x10000001);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x00008820);
        assert_eq!(data.read_u32::<NativeEndian>().unwrap(), 0x0810000a);
    }
}
