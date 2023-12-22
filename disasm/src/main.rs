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

pub fn effective_address_calculation(rm: u8) -> &'static str {
    match rm {
        0b000 => "bx + si",
        0b001 => "bx + di",
        0b010 => "bp + si",
        0b011 => "bp + di",
        0b100 => "si",
        0b101 => "di",
        0b110 => "bp",
        0b111 => "bx",
        _ => panic!("Unknown rm: {:#010b}", rm),
    }
}

pub fn emit_mov(dst: &str, src: &str) -> String {
    format!("mov {}, {}", dst, src)
}

pub fn disassembly(input: &[u8]) -> Result<String, &'static str> {
    let mut bytes_reader = BytesReader::new(input);
    let mut res = String::new();

    res.push_str("bits 16\n\n");

    while !bytes_reader.has_reached_end() {
        let op = bytes_reader.read_byte();

        if op & 0b10110000 == 0b10110000 {
            dis_mov_immediate_to_register(&mut bytes_reader, &mut res, op);
        } else if op & 0b10001000 == 0b10001000 {
            dis_mov(&mut bytes_reader, &mut res, op);
        } else if op & 0b10100010 == 0b10100010 {
            dis_mov_accumulator_to_memory(&mut bytes_reader, &mut res);
        } else if op & 0b10100000 == 0b10100000 {
            dis_mov_memory_to_accumulator(&mut bytes_reader, &mut res);
        } else if op & 0b11000110 == 0b11000110 {
            dis_mov_immediate_to_register_or_memory(&mut bytes_reader, &mut res, op);
        } else {
            panic!("Unsupported op: {:#010b}, generated: {res}", op);
        }
    }

    Ok(res)
}

pub fn dis_mov(bytes_reader: &mut BytesReader, output: &mut String, op: u8) {
    let inst_part_2 = bytes_reader.read_byte();

    let d = (op & 0b00000010) >> 1;
    let w = op & 0b00000001;

    let mod_part = (inst_part_2 & 0b11000000) >> 6;

    let reg = (inst_part_2 & 0b00111000) >> 3;
    let rm = inst_part_2 & 0b00000111;

    let effective_address = dis_effective_address(bytes_reader, rm, mod_part, w);
    let register = decode_register(reg, w);

    let mov = if d == 0b1 {
        emit_mov(register, &effective_address)
    } else {
        emit_mov(&effective_address, register)
    };

    output.push_str(&mov);
    output.push('\n');
}

pub fn dis_mov_immediate_to_register(bytes_reader: &mut BytesReader, output: &mut String, op: u8) {
    let w = (op & 0b00001000) >> 3;
    let reg = op & 0b00000111;

    let register = decode_register(reg, w);

    let data = if w == 0b1 {
        let data_low = bytes_reader.read_byte();
        let data_high = bytes_reader.read_byte();
        let bytes: [u8; 2] = [data_low, data_high];

        u16::from_le_bytes(bytes).to_string()
    } else {
        bytes_reader.read_byte().to_string()
    };

    let mov = emit_mov(register, &data);
    output.push_str(&mov);

    output.push('\n');
}

pub fn dis_mov_memory_to_accumulator(bytes_reader: &mut BytesReader, output: &mut String) {
    let data_low = bytes_reader.read_byte();
    let data_high = bytes_reader.read_byte();
    let bytes: [u8; 2] = [data_low, data_high];
    let address = u16::from_le_bytes(bytes).to_string();

    let mov = emit_mov("ax", &format!("[{address}]"));
    output.push_str(&mov);

    output.push('\n');
}

pub fn dis_mov_accumulator_to_memory(bytes_reader: &mut BytesReader, output: &mut String) {
    let data_low = bytes_reader.read_byte();
    let data_high = bytes_reader.read_byte();
    let bytes: [u8; 2] = [data_low, data_high];
    let address = u16::from_le_bytes(bytes).to_string();

    let mov = emit_mov(&format!("[{address}]"), "ax");
    output.push_str(&mov);

    output.push('\n');
}

pub fn dis_mov_immediate_to_register_or_memory(
    bytes_reader: &mut BytesReader,
    output: &mut String,
    op: u8,
) {
    let inst_part_2 = bytes_reader.read_byte();

    let w = op & 0b00000001;

    let mod_part = (inst_part_2 & 0b11000000) >> 6;
    let rm = inst_part_2 & 0b00000111;

    let effective_address = dis_effective_address(bytes_reader, rm, mod_part, 0b0);

    let value = if w == 0b1 {
        let data_low = bytes_reader.read_byte();
        let data_high = bytes_reader.read_byte();

        let bytes: [u8; 2] = [data_low, data_high];
        let data = u16::from_le_bytes(bytes);

        format!("word {data}")
    } else {
        let data = bytes_reader.read_byte();

        format!("byte {data}")
    };

    let mov = emit_mov(&effective_address, &value);
    output.push_str(&mov);
    output.push('\n');
}

pub fn dis_effective_address(
    bytes_reader: &mut BytesReader,
    rm: u8,
    mod_part: u8,
    w: u8,
) -> String {
    match mod_part {
        0b11 => String::from(decode_register(rm, w)),
        0b00 => {
            if rm == 0b110 {
                // Direct address
                let data_low = bytes_reader.read_byte();
                let data_high = bytes_reader.read_byte();

                let bytes: [u8; 2] = [data_low, data_high];
                let address = u16::from_le_bytes(bytes);

                format!("[{address}]")
            } else {
                format!("[{}]", effective_address_calculation(rm))
            }
        }
        0b01 => {
            let data = bytes_reader.read_byte() as i8;
            let mut effective_address = String::new();

            effective_address.push('[');
            effective_address.push_str(effective_address_calculation(rm));

            if data != 0 {
                if data > 0 {
                    effective_address.push_str(" + ");
                    effective_address.push_str(&data.to_string());
                } else {
                    effective_address.push_str(" - ");
                    effective_address.push_str(&(-data).to_string());
                }
            }

            effective_address.push(']');

            effective_address
        }
        0b10 => {
            let data_low = bytes_reader.read_byte();
            let data_high = bytes_reader.read_byte();

            let bytes: [u8; 2] = [data_low, data_high];
            let data = i16::from_le_bytes(bytes);

            let mut effective_address = String::new();

            effective_address.push('[');
            effective_address.push_str(effective_address_calculation(rm));

            if data != 0 {
                if data > 0 {
                    effective_address.push_str(" + ");
                    effective_address.push_str(&data.to_string());
                } else {
                    effective_address.push_str(" - ");
                    effective_address.push_str(&(-data).to_string());
                }
            }

            effective_address.push(']');

            effective_address
        }
        _ => panic!("Unsupported mod: {:#010b}", mod_part),
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::*;

    fn do_test(file_name: &str) {
        let binary_path = PathBuf::from("test_resources").join(file_name);
        let binary = fs::read(binary_path).unwrap();

        let asm_path = PathBuf::from("test_resources").join(format!("{file_name}.asm"));
        let asm = fs::read_to_string(asm_path).unwrap();
        let generated_asm = disassembly(&binary).unwrap();

        assert_eq!(generated_asm, asm);
    }

    #[test]
    fn single_register_mov() {
        do_test("single_register_mov");
    }

    #[test]
    fn many_register_mov() {
        do_test("many_register_mov");
    }

    #[test]
    fn source_address_calculation() {
        do_test("source_address_calculation");
    }

    #[test]
    fn source_address_calculation_plus_8_bit_displacement() {
        do_test("source_address_calculation_plus_8_bit_displacement");
    }

    #[test]
    fn source_address_calculation_plus_16_bit_displacement() {
        do_test("source_address_calculation_plus_16_bit_displacement");
    }

    #[test]
    fn dest_address_calculation() {
        do_test("dest_address_calculation");
    }

    #[test]
    fn immediate_to_register_8_bit() {
        do_test("immediate_to_register_8_bit");
    }

    #[test]
    fn immediate_to_register_16_bit() {
        do_test("immediate_to_register_16_bit");
    }

    #[test]
    fn signed_displacements() {
        do_test("signed_displacements");
    }

    #[test]
    fn direct_address() {
        do_test("direct_address");
    }

    #[test]
    fn memory_to_accumulator() {
        do_test("memory_to_accumulator");
    }

    #[test]
    fn accumulator_to_memory() {
        do_test("accumulator_to_memory");
    }

    #[test]
    fn explicit_sizes() {
        do_test("explicit_sizes");
    }
}
