use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct RegisterName(u8);

impl RegisterName {
    pub fn new(val: u8) -> Self {
        assert!(val < 32);
        RegisterName(val)
    }

    pub fn num(&self) -> u8 {
        self.0
    }

    pub fn try_from_num(val: u8) -> Option<Self> {
        if val < 32 {
            Some(RegisterName(val))
        } else {
            None
        }
    }

    pub fn try_from_name(name: &str) -> Option<Self> {
        // Try parsing as numeric form (e.g. 13)
        if let Ok(x) = u8::from_str(name) {
            return Self::try_from_num(x);
        }

        // Try parsing as textual form (e.g. $v1)
        let val = match name {
            "r0" | "zero" => 0,
            "at" => 1,
            "v0" => 2,
            "a0" => 4,
            "a1" => 5,
            "a2" => 6,
            "a3" => 7,
            "t0" => 8,
            "t1" => 9,
            "t2" => 10,
            "t3" => 11,
            "t4" => 12,
            "t5" => 13,
            "t6" => 14,
            "t7" => 15,
            "s0" => 16,
            "s1" => 17,
            "s2" => 18,
            "s3" => 19,
            "s4" => 20,
            "s5" => 21,
            "s6" => 22,
            "s7" => 23,
            "t8" => 24,
            "t9" => 25,
            "k0" => 26,
            "k1" => 27,
            "gp" => 28,
            "sp" => 29,
            "s8" => 30,
            "ra" => 31,
            _ => return None,
        };

        RegisterName::try_from_num(val)
    }
}
