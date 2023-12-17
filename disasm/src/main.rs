pub struct BytesReader<'a> {
    data: &'a [u8],
    cursor: usize,
}

impl<'a> BytesReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, cursor: 0 }
    }

    pub fn read_byte(&mut self) -> u8 {
        if self.has_reached_end() {
            panic!("Has reached end");
        }

        let res = self.data[self.cursor];
        self.cursor += 1;

        res
    }

    pub fn has_reached_end(&self) -> bool {
        self.cursor >= self.data.len()
    }
}

const MOV_MASK: u8 = 0b10001000;

pub fn decode_register(data: u8, w: u8) -> &'static str {
    match data {
        0b000 if w == 0b0 => "al",
        0b000 if w == 0b1 => "ax",
        0b001 if w == 0b0 => "cl",
        0b001 if w == 0b1 => "cx",
        0b010 if w == 0b0 => "dl",
        0b010 if w == 0b1 => "dx",
        0b011 if w == 0b0 => "bl",
        0b011 if w == 0b1 => "bx",
        0b100 if w == 0b0 => "ah",
        0b100 if w == 0b1 => "sp",
        0b101 if w == 0b0 => "ch",
        0b101 if w == 0b1 => "bp",
        0b110 if w == 0b0 => "dh",
        0b110 if w == 0b1 => "si",
        0b111 if w == 0b0 => "bh",
        0b111 if w == 0b1 => "di",
        _ => panic!("Unknown register: reg: {:#010b}, w: {:#010b}", data, w),
    }
}

pub fn disassembly(input: &[u8]) -> Result<String, &'static str> {
    let mut bytes_reader = BytesReader::new(input);
    let mut res = String::new();

    res.push_str("bits 16\n\n");

    while !bytes_reader.has_reached_end() {
        let inst_part_1 = bytes_reader.read_byte();

        if inst_part_1 & MOV_MASK == MOV_MASK {
            let inst_part_2 = bytes_reader.read_byte();

            let d = (inst_part_1 & 0b00000010) >> 1;
            let w = inst_part_1 & 0b00000001;

            if d != 0b00000000 {
                panic!("Failed to disasseble");
            }

            let mod_part = (inst_part_2 & 0b11000000) >> 6;

            if mod_part != 0b11 {
                panic!("Unsupported MOV mod value");
            }

            let reg = (inst_part_2 & 0b00111000) >> 3;
            let rm = inst_part_2 & 0b00000111;

            res.push_str("mov ");
            res.push_str(&format!("{}, ", decode_register(rm, w)));
            res.push_str(decode_register(reg, w));
            res.push('\n');
        }
    }

    Ok(res)
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn single_register_mov() {
        let binary = fs::read("test_resources/listing_0037_single_register_mov").unwrap();
        let asm =
            fs::read_to_string("test_resources/listing_0037_single_register_mov.asm").unwrap();
        let generated_asm = disassembly(&binary).unwrap();

        assert_eq!(generated_asm, asm);
    }

    #[test]
    fn many_register_mov() {
        let binary = fs::read("test_resources/listing_0038_many_register_mov").unwrap();
        let asm =
            fs::read_to_string("test_resources/listing_0038_many_register_mov.asm").unwrap();
        let generated_asm = disassembly(&binary).unwrap();

        assert_eq!(generated_asm, asm);
    }
}
