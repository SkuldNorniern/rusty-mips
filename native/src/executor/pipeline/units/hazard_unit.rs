use crate::executor::pipeline::pipes;

#[derive(Copy, Clone, Debug, Default)]
pub struct HazardUnit {
    pub redo_if: bool,
    pub drop_if: bool,
}

pub fn hazard_ctrl(
    if_output: &pipes::IfPipe,
    id_output: &pipes::IdPipe,
    ex_output: &pipes::ExPipe,
) -> HazardUnit {
    let mut ret: HazardUnit = Default::default();

    let ins = if_output.inst;
    let opcode = (ins & 0xFC000000) >> 26;
    let rs = (ins & 0x03E00000) >> 21;
    let rt = (ins & 0x001F0000) >> 16;

    let reads_rs_rt = opcode == 0 || opcode == 4 || opcode == 0x2b;
    let reads_rs = opcode == 0x23;

    // opcode is beq
    if opcode == 4 {
        let id_reg_write = if id_output.ctr_unit.reg_dst {
            id_output.rd
        } else {
            id_output.rt
        };
        if id_output.ctr_unit.reg_write && id_reg_write != 0 {
            if (reads_rs_rt && (rs == id_reg_write || rt == id_reg_write))
                || (reads_rs && rs == id_reg_write)
            {
                // calc-branch OR load-branch
                ret.redo_if = true;
            }
        }
        if ex_output.ctr_unit.mem_read && ex_output.rd != 0 {
            if (reads_rs_rt && (rs == ex_output.rd || rt == ex_output.rd))
                || (reads_rs && rs == ex_output.rd)
            {
                // load-branch
                ret.redo_if = true;
            }
        }
    }

    // opcode not sw
    if opcode != 0x2b && id_output.ctr_unit.mem_read && id_output.rt != 0 {
        if (reads_rs_rt && (rs == id_output.rt || rt == id_output.rt))
            || (reads_rs && rs == id_output.rt)
        {
            // load-use
            ret.redo_if = true;
        }
    }

    if id_output.branch_taken || id_output.ctr_unit.jump {
        ret.drop_if = true;
        println!("Hazard: branch taken");
    }

    ret
}
