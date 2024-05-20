use crate::system::cpu::CPU;

// Define the macro
macro_rules! illegal_op {
    ($opcode:expr) => {
        Instruction {
            name: "???",
            opcode_num: $opcode,
            addr_mode: AddressingMode::Implied,
            addr_func: |_| { (OpcodeData{data: None, address: None, offset: None}, 0) },
            func: xxx,
            base_clocks: 2,
            bytes: 1,
        }
    };
}

// This instruction isn't real. It cannot ever happen, so it's used as a sort
// of placeholder or "We haven't started the program yet." It's the initial
// instruction the CPU is loaded with
pub const DEFAULT_ILLEGAL_OP: Instruction = illegal_op!(0x00);

/// A large lookup table for each instruction, indexed by that 
/// instruction's opcode. The full table can be viewed here:
/// https://www.masswerk.at/6502/6502_instruction_set.html
pub const INSTRUCTION_TABLE: [Instruction; 256] = [
    Instruction{name: "BRK", opcode_num: 0x00, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: brk, base_clocks: 7, bytes: 2},
	Instruction{name: "ORA", opcode_num: 0x01, addr_mode: AddressingMode::IndirectX, addr_func: indirect_x, func: ora, base_clocks: 6, bytes: 3},
	illegal_op!(0x02),
	illegal_op!(0x03),
	illegal_op!(0x04),
	Instruction{name: "ORA", opcode_num: 0x05, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: ora, base_clocks: 3, bytes: 2},
	Instruction{name: "ASL", opcode_num: 0x06, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: asl, base_clocks: 5, bytes: 2},
	illegal_op!(0x07),
	Instruction{name: "PHP", opcode_num: 0x08, addr_mode: AddressingMode::Implied, addr_func: implied, func: php, base_clocks: 3, bytes: 1},
	Instruction{name: "ORA", opcode_num: 0x09, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: ora, base_clocks: 2, bytes: 2},
	Instruction{name: "ASL", opcode_num: 0x0A, addr_mode: AddressingMode::Accumulator, addr_func: accumulator, func: asl, base_clocks: 2, bytes: 1},
	illegal_op!(0x0B),
	illegal_op!(0x0C),
	Instruction{name: "ORA", opcode_num: 0x0D, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: ora, base_clocks: 4, bytes: 3},
	Instruction{name: "ASL", opcode_num: 0x0E, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: asl, base_clocks: 6, bytes: 3},
	illegal_op!(0x0F),

	Instruction{name: "BPL", opcode_num: 0x10, addr_mode: AddressingMode::Relative, addr_func: relative, func: bpl, base_clocks: 2, bytes: 2},
	Instruction{name: "ORA", opcode_num: 0x11, addr_mode: AddressingMode::IndirectY, addr_func: indirect_y, func: ora, base_clocks: 5, bytes: 3},
	illegal_op!(0x12),
	illegal_op!(0x13),
	illegal_op!(0x14),
	Instruction{name: "ORA", opcode_num: 0x15, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: ora, base_clocks: 4, bytes: 2},
	Instruction{name: "ASL", opcode_num: 0x16, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: asl, base_clocks: 6, bytes: 2},
	illegal_op!(0x17),
	Instruction{name: "CLC", opcode_num: 0x18, addr_mode: AddressingMode::Implied, addr_func: implied, func: clc, base_clocks: 2, bytes: 1},
	Instruction{name: "ORA", opcode_num: 0x19, addr_mode: AddressingMode::AbsoluteY, addr_func: absolute_y, func: ora, base_clocks: 4, bytes: 3},
	illegal_op!(0x1A),
	illegal_op!(0x1B),
	illegal_op!(0x1C),
	Instruction{name: "ORA", opcode_num: 0x1D, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: ora, base_clocks: 4, bytes: 3},
	Instruction{name: "ASL", opcode_num: 0x1E, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: asl, base_clocks: 7, bytes: 3},
	illegal_op!(0x1F),

	Instruction{name: "JSR", opcode_num: 0x20, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: jsr, base_clocks: 6, bytes: 3},
	Instruction{name: "AND", opcode_num: 0x21, addr_mode: AddressingMode::IndirectX, addr_func: indirect_x, func: and, base_clocks: 6, bytes: 3},
	illegal_op!(0x22),
	illegal_op!(0x23),
	Instruction{name: "BIT", opcode_num: 0x24, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: bit, base_clocks: 3, bytes: 2},
	Instruction{name: "AND", opcode_num: 0x25, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: and, base_clocks: 3, bytes: 2},
	Instruction{name: "ROL", opcode_num: 0x26, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: rol, base_clocks: 5, bytes: 2},
	illegal_op!(0x27),
	Instruction{name: "PLP", opcode_num: 0x28, addr_mode: AddressingMode::Implied, addr_func: implied, func: plp, base_clocks: 4, bytes: 1},
	Instruction{name: "AND", opcode_num: 0x29, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: and, base_clocks: 2, bytes: 2},
	Instruction{name: "ROL", opcode_num: 0x2A, addr_mode: AddressingMode::Implied, addr_func: implied, func: rol, base_clocks: 2, bytes: 1},
	illegal_op!(0x2B),
	Instruction{name: "BIT", opcode_num: 0x2C, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: bit, base_clocks: 4, bytes: 3},
	Instruction{name: "AND", opcode_num: 0x2D, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: and, base_clocks: 4, bytes: 3},
	Instruction{name: "ROL", opcode_num: 0x2E, addr_mode: AddressingMode::Accumulator, addr_func: accumulator, func: rol, base_clocks: 6, bytes: 1},
	illegal_op!(0x2F),

	Instruction{name: "BMI", opcode_num: 0x30, addr_mode: AddressingMode::Relative, addr_func: relative, func: bmi, base_clocks: 2, bytes: 2},
	Instruction{name: "AND", opcode_num: 0x31, addr_mode: AddressingMode::IndirectY, addr_func: indirect_y, func: and, base_clocks: 5, bytes: 3},
	illegal_op!(0x32),
	illegal_op!(0x33),
	illegal_op!(0x34),
	Instruction{name: "AND", opcode_num: 0x35, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: and, base_clocks: 4, bytes: 2},
	Instruction{name: "ROL", opcode_num: 0x36, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: rol, base_clocks: 6, bytes: 2},
	illegal_op!(0x37),
	Instruction{name: "SEC", opcode_num: 0x38, addr_mode: AddressingMode::Implied, addr_func: implied, func: sec, base_clocks: 2, bytes: 1},
	Instruction{name: "AND", opcode_num: 0x39, addr_mode: AddressingMode::AbsoluteY, addr_func: absolute_y, func: and, base_clocks: 4, bytes: 3},
	illegal_op!(0x3A),
	illegal_op!(0x3B),
	illegal_op!(0x3C),
	Instruction{name: "AND", opcode_num: 0x3D, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: and, base_clocks: 4, bytes: 3},
	Instruction{name: "ROL", opcode_num: 0x3E, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: rol, base_clocks: 7, bytes: 3},
	illegal_op!(0x3F),


	Instruction{name: "RTI", opcode_num: 0x40, addr_mode: AddressingMode::Implied, addr_func: implied, func: rti, base_clocks: 6, bytes: 1},
	Instruction{name: "EOR", opcode_num: 0x41, addr_mode: AddressingMode::IndirectX, addr_func: indirect_x, func: eor, base_clocks: 6, bytes: 3},
	illegal_op!(0x42),
	illegal_op!(0x43),
	illegal_op!(0x44),
	Instruction{name: "EOR", opcode_num: 0x45, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: eor, base_clocks: 3, bytes: 2},
	Instruction{name: "LSR", opcode_num: 0x46, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: lsr, base_clocks: 5, bytes: 2},
	illegal_op!(0x47),
	Instruction{name: "PHA", opcode_num: 0x48, addr_mode: AddressingMode::Implied, addr_func: implied, func: pha, base_clocks: 3, bytes: 1},
	Instruction{name: "EOR", opcode_num: 0x49, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: eor, base_clocks: 2, bytes: 2},
	Instruction{name: "LSR", opcode_num: 0x4A, addr_mode: AddressingMode::Accumulator, addr_func: accumulator, func: lsr, base_clocks: 2, bytes: 1},
	illegal_op!(0x4B),
	Instruction{name: "JMP", opcode_num: 0x4C, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: jmp, base_clocks: 3, bytes: 3},
	Instruction{name: "EOR", opcode_num: 0x4D, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: eor, base_clocks: 4, bytes: 3},
	Instruction{name: "LSR", opcode_num: 0x4E, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: lsr, base_clocks: 6, bytes: 3},
	illegal_op!(0x4F),

	Instruction{name: "BVC", opcode_num: 0x50, addr_mode: AddressingMode::Relative, addr_func: relative, func: bvc, base_clocks: 2, bytes: 2},
	Instruction{name: "EOR", opcode_num: 0x51, addr_mode: AddressingMode::IndirectY, addr_func: indirect_y, func: eor, base_clocks: 5, bytes: 3},
	illegal_op!(0x52),
	illegal_op!(0x53),
	illegal_op!(0x54),
	Instruction{name: "EOR", opcode_num: 0x55, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: eor, base_clocks: 4, bytes: 2},
	Instruction{name: "LSR", opcode_num: 0x56, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: lsr, base_clocks: 6, bytes: 2},
	illegal_op!(0x57),
	Instruction{name: "CLI", opcode_num: 0x58, addr_mode: AddressingMode::Implied, addr_func: implied, func: cli, base_clocks: 2, bytes: 1},
	Instruction{name: "EOR", opcode_num: 0x59, addr_mode: AddressingMode::AbsoluteY, addr_func: absolute_y, func: eor, base_clocks: 4, bytes: 3},
	illegal_op!(0x5A),
	illegal_op!(0x5B),
	illegal_op!(0x5C),
	Instruction{name: "EOR", opcode_num: 0x5D, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: eor, base_clocks: 4, bytes: 3},
	Instruction{name: "LSR", opcode_num: 0x5E, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: lsr, base_clocks: 7, bytes: 3},
	illegal_op!(0x5F),

	Instruction{name: "RTS", opcode_num: 0x60, addr_mode: AddressingMode::Implied, addr_func: implied, func: rts, base_clocks: 6, bytes: 1},
	Instruction{name: "ADC", opcode_num: 0x61, addr_mode: AddressingMode::IndirectX, addr_func: indirect_x, func: adc, base_clocks: 6, bytes: 3},
	illegal_op!(0x62),
	illegal_op!(0x63),
	illegal_op!(0x64),
	Instruction{name: "ADC", opcode_num: 0x65, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: adc, base_clocks: 3, bytes: 2},
	Instruction{name: "ROR", opcode_num: 0x66, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: ror, base_clocks: 5, bytes: 2},
	illegal_op!(0x67),
	Instruction{name: "PLA", opcode_num: 0x68, addr_mode: AddressingMode::Implied, addr_func: implied, func: pla, base_clocks: 4, bytes: 1},
	Instruction{name: "ADC", opcode_num: 0x69, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: adc, base_clocks: 2, bytes: 2},
	Instruction{name: "ROR", opcode_num: 0x6A, addr_mode: AddressingMode::Accumulator, addr_func: accumulator, func: ror, base_clocks: 2, bytes: 1},
	illegal_op!(0x6B),
	Instruction{name: "JMP", opcode_num: 0x6C, addr_mode: AddressingMode::Indirect, addr_func: indirect, func: jmp, base_clocks: 5, bytes: 3},
	Instruction{name: "ADC", opcode_num: 0x6D, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: adc, base_clocks: 4, bytes: 3},
	Instruction{name: "ROR", opcode_num: 0x6E, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: ror, base_clocks: 6, bytes: 3},
	illegal_op!(0x6F),

	Instruction{name: "BVS", opcode_num: 0x70, addr_mode: AddressingMode::Relative, addr_func: relative, func: bvs, base_clocks: 2, bytes: 2},
	Instruction{name: "ADC", opcode_num: 0x71, addr_mode: AddressingMode::IndirectY, addr_func: indirect_y, func: adc, base_clocks: 5, bytes: 3},
	illegal_op!(0x72),
	illegal_op!(0x73),
	illegal_op!(0x74),
	Instruction{name: "ADC", opcode_num: 0x75, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: adc, base_clocks: 4, bytes: 2},
	Instruction{name: "ROR", opcode_num: 0x76, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: ror, base_clocks: 6, bytes: 2},
	illegal_op!(0x77),
	Instruction{name: "SEI", opcode_num: 0x78, addr_mode: AddressingMode::Implied, addr_func: implied, func: sei, base_clocks: 2, bytes: 1},
	Instruction{name: "ADC", opcode_num: 0x79, addr_mode: AddressingMode::AbsoluteY, addr_func: absolute_y, func: adc, base_clocks: 4, bytes: 3},
	illegal_op!(0x7A),
	illegal_op!(0x7B),
	illegal_op!(0x7C),
	Instruction{name: "ADC", opcode_num: 0x7D, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: adc, base_clocks: 4, bytes: 3},
	Instruction{name: "ROR", opcode_num: 0x7E, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: ror, base_clocks: 7, bytes: 3},
	illegal_op!(0x7F),


	illegal_op!(0x80),
	Instruction{name: "STA", opcode_num: 0x81, addr_mode: AddressingMode::IndirectX, addr_func: indirect_x, func: sta, base_clocks: 6, bytes: 3},
	illegal_op!(0x82),
	illegal_op!(0x83),
	Instruction{name: "STY", opcode_num: 0x84, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: sty, base_clocks: 3, bytes: 2},
	Instruction{name: "STA", opcode_num: 0x85, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: sta, base_clocks: 3, bytes: 2},
	Instruction{name: "STX", opcode_num: 0x86, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: stx, base_clocks: 3, bytes: 2},
	illegal_op!(0x87),
	Instruction{name: "DEY", opcode_num: 0x88, addr_mode: AddressingMode::Implied, addr_func: implied, func: dey, base_clocks: 2, bytes: 1},
	illegal_op!(0x89),
	Instruction{name: "TXA", opcode_num: 0x8A, addr_mode: AddressingMode::Implied, addr_func: implied, func: txa, base_clocks: 2, bytes: 1},
	illegal_op!(0x8B),
	Instruction{name: "STY", opcode_num: 0x8C, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: sty, base_clocks: 4, bytes: 3},
	Instruction{name: "STA", opcode_num: 0x8D, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: sta, base_clocks: 4, bytes: 3},
	Instruction{name: "STX", opcode_num: 0x8E, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: stx, base_clocks: 4, bytes: 3},
	illegal_op!(0x8F),

	Instruction{name: "BCC", opcode_num: 0x90, addr_mode: AddressingMode::Relative, addr_func: relative, func: bcc, base_clocks: 2, bytes: 2},
	Instruction{name: "STA", opcode_num: 0x91, addr_mode: AddressingMode::IndirectY, addr_func: indirect_y, func: sta, base_clocks: 6, bytes: 3},
	illegal_op!(0x92),
	illegal_op!(0x93),
	Instruction{name: "STY", opcode_num: 0x94, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: sty, base_clocks: 4, bytes: 2},
	Instruction{name: "STA", opcode_num: 0x95, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: sta, base_clocks: 4, bytes: 2},
	Instruction{name: "STX", opcode_num: 0x96, addr_mode: AddressingMode::ZeroPageY, addr_func: zpage_y, func: stx, base_clocks: 4, bytes: 2},
	illegal_op!(0x97),
	Instruction{name: "TYA", opcode_num: 0x98, addr_mode: AddressingMode::Implied, addr_func: implied, func: tya, base_clocks: 2, bytes: 1},
	Instruction{name: "STA", opcode_num: 0x99, addr_mode: AddressingMode::AbsoluteY, addr_func: absolute_y, func: sta, base_clocks: 5, bytes: 3},
	Instruction{name: "TXS", opcode_num: 0x9A, addr_mode: AddressingMode::Implied, addr_func: implied, func: txs, base_clocks: 2, bytes: 1},
	illegal_op!(0x9B),
	illegal_op!(0x9C),
	Instruction{name: "STA", opcode_num: 0x9D, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: sta, base_clocks: 5, bytes: 3},
	illegal_op!(0x9E),
	illegal_op!(0x9F),

	Instruction{name: "LDY", opcode_num: 0xA0, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: ldy, base_clocks: 2, bytes: 2},
	Instruction{name: "LDA", opcode_num: 0xA1, addr_mode: AddressingMode::IndirectX, addr_func: indirect_x, func: lda, base_clocks: 6, bytes: 3},
	Instruction{name: "LDX", opcode_num: 0xA2, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: ldx, base_clocks: 2, bytes: 2},
	illegal_op!(0xA3),
	Instruction{name: "LDY", opcode_num: 0xA4, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: ldy, base_clocks: 3, bytes: 2},
	Instruction{name: "LDA", opcode_num: 0xA5, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: lda, base_clocks: 3, bytes: 2},
	Instruction{name: "LDX", opcode_num: 0xA6, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: ldx, base_clocks: 3, bytes: 2},
	illegal_op!(0xA7),
	Instruction{name: "TAY", opcode_num: 0xA8, addr_mode: AddressingMode::Implied, addr_func: implied, func: tay, base_clocks: 2, bytes: 1},
	Instruction{name: "LDA", opcode_num: 0xA9, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: lda, base_clocks: 2, bytes: 2},
	Instruction{name: "TAX", opcode_num: 0xAA, addr_mode: AddressingMode::Implied, addr_func: implied, func: tax, base_clocks: 2, bytes: 1},
	illegal_op!(0xAB),
	Instruction{name: "LDY", opcode_num: 0xAC, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: ldy, base_clocks: 4, bytes: 3},
	Instruction{name: "LDA", opcode_num: 0xAD, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: lda, base_clocks: 4, bytes: 3},
	Instruction{name: "LDX", opcode_num: 0xAE, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: ldx, base_clocks: 4, bytes: 3},
	illegal_op!(0xAF),

	Instruction{name: "BCS", opcode_num: 0xB0, addr_mode: AddressingMode::Relative, addr_func: relative, func: bcs, base_clocks: 2, bytes: 2},
	Instruction{name: "LDA", opcode_num: 0xB1, addr_mode: AddressingMode::IndirectY, addr_func: indirect_y, func: lda, base_clocks: 5, bytes: 3},
	illegal_op!(0xB2),
	illegal_op!(0xB3),
	Instruction{name: "LDY", opcode_num: 0xB4, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: ldy, base_clocks: 4, bytes: 2},
	Instruction{name: "LDA", opcode_num: 0xB5, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: lda, base_clocks: 4, bytes: 2},
	Instruction{name: "LDX", opcode_num: 0xB6, addr_mode: AddressingMode::ZeroPageY, addr_func: zpage_y, func: ldx, base_clocks: 4, bytes: 2},
	illegal_op!(0xB7),
	Instruction{name: "CLV", opcode_num: 0xB8, addr_mode: AddressingMode::Implied, addr_func: implied, func: clv, base_clocks: 2, bytes: 1},
	Instruction{name: "LDA", opcode_num: 0xB9, addr_mode: AddressingMode::AbsoluteY, addr_func: absolute_y, func: lda, base_clocks: 4, bytes: 3},
	Instruction{name: "TSX", opcode_num: 0xBA, addr_mode: AddressingMode::Implied, addr_func: implied, func: tsx, base_clocks: 2, bytes: 1},
	illegal_op!(0xBB),
	Instruction{name: "LDY", opcode_num: 0xBC, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: ldy, base_clocks: 4, bytes: 3},
	Instruction{name: "LDA", opcode_num: 0xBD, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: lda, base_clocks: 4, bytes: 3},
	Instruction{name: "LDX", opcode_num: 0xBE, addr_mode: AddressingMode::AbsoluteY, addr_func: absolute_y, func: ldx, base_clocks: 4, bytes: 3},
	illegal_op!(0xBF),


	Instruction{name: "CPY", opcode_num: 0xC0, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: cpy, base_clocks: 2, bytes: 2},
	Instruction{name: "CMP", opcode_num: 0xC1, addr_mode: AddressingMode::IndirectX, addr_func: indirect_x, func: cmp, base_clocks: 6, bytes: 3},
	illegal_op!(0xC2),
	illegal_op!(0xC3),
	Instruction{name: "CPY", opcode_num: 0xC4, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: cpy, base_clocks: 3, bytes: 2},
	Instruction{name: "CMP", opcode_num: 0xC5, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: cmp, base_clocks: 3, bytes: 2},
	Instruction{name: "DEC", opcode_num: 0xC6, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: dec, base_clocks: 5, bytes: 2},
	illegal_op!(0xC7),
	Instruction{name: "INY", opcode_num: 0xC8, addr_mode: AddressingMode::Implied, addr_func: implied, func: iny, base_clocks: 2, bytes: 1},
	Instruction{name: "CMP", opcode_num: 0xC9, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: cmp, base_clocks: 2, bytes: 2},
	Instruction{name: "DEX", opcode_num: 0xCA, addr_mode: AddressingMode::Implied, addr_func: implied, func: dex, base_clocks: 2, bytes: 1},
	illegal_op!(0xCB),
	Instruction{name: "CPY", opcode_num: 0xCC, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: cpy, base_clocks: 4, bytes: 3},
	Instruction{name: "CMP", opcode_num: 0xCD, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: cmp, base_clocks: 4, bytes: 3},
	Instruction{name: "DEC", opcode_num: 0xCE, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: dec, base_clocks: 6, bytes: 3},
	illegal_op!(0xCF),

	Instruction{name: "BNE", opcode_num: 0xD0, addr_mode: AddressingMode::Relative, addr_func: relative, func: bne, base_clocks: 2, bytes: 2},
	Instruction{name: "CMP", opcode_num: 0xD1, addr_mode: AddressingMode::IndirectY, addr_func: indirect_y, func: cmp, base_clocks: 5, bytes: 3},
	illegal_op!(0xD2),
	illegal_op!(0xD3),
	illegal_op!(0xD4),
	Instruction{name: "CMP", opcode_num: 0xD5, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: cmp, base_clocks: 4, bytes: 2},
	Instruction{name: "DEC", opcode_num: 0xD6, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: dec, base_clocks: 6, bytes: 2},
	illegal_op!(0xD7),
	Instruction{name: "CLD", opcode_num: 0xD8, addr_mode: AddressingMode::Implied, addr_func: implied, func: cld, base_clocks: 2, bytes: 1},
	Instruction{name: "CMP", opcode_num: 0xD9, addr_mode: AddressingMode::AbsoluteY, addr_func: absolute_y, func: cmp, base_clocks: 4, bytes: 3},
	Instruction{name: "NOP", opcode_num: 0xDA, addr_mode: AddressingMode::Implied, addr_func: implied, func: nop, base_clocks: 2, bytes: 1},
	illegal_op!(0xDB),
	illegal_op!(0xDC),
	Instruction{name: "CMP", opcode_num: 0xDD, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: cmp, base_clocks: 4, bytes: 3},
	Instruction{name: "DEC", opcode_num: 0xDE, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: dec, base_clocks: 7, bytes: 3},
	illegal_op!(0xDF),

	Instruction{name: "CPX", opcode_num: 0xE0, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: cpx, base_clocks: 2, bytes: 2},
	Instruction{name: "SBC", opcode_num: 0xE1, addr_mode: AddressingMode::IndirectX, addr_func: indirect_x, func: sbc, base_clocks: 6, bytes: 3},
	illegal_op!(0xE2),
	illegal_op!(0xE3),
	Instruction{name: "CPX", opcode_num: 0xE4, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: cpx, base_clocks: 3, bytes: 2},
	Instruction{name: "SBC", opcode_num: 0xE5, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: sbc, base_clocks: 3, bytes: 2},
	Instruction{name: "INC", opcode_num: 0xE6, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: inc, base_clocks: 5, bytes: 2},
	illegal_op!(0xE7),
	Instruction{name: "INX", opcode_num: 0xE8, addr_mode: AddressingMode::Implied, addr_func: implied, func: inx, base_clocks: 2, bytes: 1},
	Instruction{name: "SBC", opcode_num: 0xE9, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: sbc, base_clocks: 2, bytes: 2},
	Instruction{name: "NOP", opcode_num: 0xEA, addr_mode: AddressingMode::Implied, addr_func: implied, func: nop, base_clocks: 2, bytes: 1},
	illegal_op!(0xEB),
	Instruction{name: "CPX", opcode_num: 0xEC, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: cpx, base_clocks: 4, bytes: 3},
	Instruction{name: "SBC", opcode_num: 0xED, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: sbc, base_clocks: 4, bytes: 3},
	Instruction{name: "INC", opcode_num: 0xEE, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: inc, base_clocks: 6, bytes: 3},
	illegal_op!(0xEF),

	Instruction{name: "BEQ", opcode_num: 0xF0, addr_mode: AddressingMode::Relative, addr_func: relative, func: beq, base_clocks: 2, bytes: 2},
	Instruction{name: "SBC", opcode_num: 0xF1, addr_mode: AddressingMode::IndirectY, addr_func: indirect_y, func: sbc, base_clocks: 5, bytes: 3},
	illegal_op!(0xF2),
	illegal_op!(0xF3),
	illegal_op!(0xF4),
	Instruction{name: "SBC", opcode_num: 0xF5, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: sbc, base_clocks: 4, bytes: 2},
	Instruction{name: "INC", opcode_num: 0xF6, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: inc, base_clocks: 6, bytes: 2},
	illegal_op!(0xF7),
	Instruction{name: "SED", opcode_num: 0xF8, addr_mode: AddressingMode::Implied, addr_func: implied, func: sed, base_clocks: 2, bytes: 1},
	Instruction{name: "SBC", opcode_num: 0xF9, addr_mode: AddressingMode::AbsoluteY, addr_func: absolute_y, func: sbc, base_clocks: 4, bytes: 3},
	Instruction{name: "NOP", opcode_num: 0xFA, addr_mode: AddressingMode::Implied, addr_func: implied, func: nop, base_clocks: 2, bytes: 1},
	illegal_op!(0xFB),
	illegal_op!(0xFC),
	Instruction{name: "SBC", opcode_num: 0xFD, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: sbc, base_clocks: 4, bytes: 3},
	Instruction{name: "INC", opcode_num: 0xFE, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: inc, base_clocks: 7, bytes: 3},
	illegal_op!(0xFF),
];

#[derive(Clone, Copy)]
pub enum AddressingMode {
    Accumulator,
    Implied,
    Immediate,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Indirect,
    IndirectX,
    IndirectY,
    Relative,
}

#[derive(Clone, Copy)]
/// A struct containing all of the data any instruction may need for its
/// execution. Some instructions need data, some need addresses, some need both.
/// Some instructions have multiple versions, and they decide which version to
/// execute depending on whether the given OpcodeData contains an address and
/// data. 
pub struct OpcodeData {
    pub data: Option<u8>,
    pub address: Option<u16>,
    pub offset: Option<i8>,
}

#[derive(Clone)]
/// Contains all information needed for execution and debugging of each 
/// individual instruction. All fields are public for ease of use.
pub struct Instruction {
    pub name: &'static str,
    pub opcode_num: u8,
    pub addr_mode: AddressingMode,
    pub addr_func: fn(cpu: &CPU) -> (OpcodeData, usize),
    pub func: fn(cpu: &mut CPU, opcode_data: OpcodeData) -> usize,
    pub base_clocks: usize,
    pub bytes: usize,
}


// ADDRESSING MODES - Fetches data from the bus
// Returns:
//  - Data needed for opcode with this addressing mode wrapped in the OpcodeData struct
//  - Number of extra CPU cycles taken to get data
//
// Note: Implied addressing mode has no return type because no data is needed for instructions
// with implied addressing mode.
fn accumulator(cpu: &CPU) -> (OpcodeData, usize) {
    (OpcodeData{ data: Some(cpu.get_acc()), address: None, offset: None }, 0)
}
// Implied - no extra data needed for this instruction, read no extra bytes
fn implied(_: &CPU) -> (OpcodeData, usize) {
    (OpcodeData{ data: None, address: None, offset: None }, 0)
}
// Immediate - data immediatly follows instruction
fn immediate(cpu: &CPU) -> (OpcodeData, usize) {
    let data = cpu.read(cpu.get_pc() + 1);

    (OpcodeData{ data: Some(data), address: None, offset: None }, 0)
}
// Absolute - The next 2 bytes are the address in memory of the data to retrieve
fn absolute(cpu: &CPU) -> (OpcodeData, usize) {
    let abs_address = cpu.read_word(cpu.get_pc() + 1);
    let data = cpu.read(abs_address);
    
    (OpcodeData{ data: Some(data), address: Some(abs_address), offset: None }, 0)
}
// Indexed Addressing (X) - Like Absolute, but adds the x register to the abs address to get
// the "effective address," and uses that to fetch data from memory.
// Also known as Absolute X addressing.
fn absolute_x(cpu: &CPU) -> (OpcodeData, usize) {
    let abs_address = cpu.read_word(cpu.get_pc() + 1);
    let effective_address = abs_address + cpu.get_x_reg() as u16;
    let data = cpu.read(effective_address);

    let page_boundary_crossed: bool = (abs_address & 0xFF00) != (effective_address & 0xFF00);

    (
        OpcodeData{ 
            data: Some(data), 
            address: Some(effective_address), 
            offset: None 
        },
        if page_boundary_crossed { 1 } else { 0 }
    )
}
// Indexed Addressing (Y) - Like same as Indexed x, but used the y register instead.
// Also known as Absolute Y addressing.
fn absolute_y(cpu: &CPU) -> (OpcodeData, usize) {
    let abs_address = cpu.read_word(cpu.get_pc() + 1);
    let effective_address = abs_address + cpu.get_y_reg() as u16;
    let data = cpu.read(effective_address);

    let page_boundary_crossed: bool = (abs_address & 0xFF00) != (effective_address & 0xFF00);

    (
        OpcodeData{ 
            data: Some(data), 
            address: Some(effective_address), 
            offset: None 
        },
        if page_boundary_crossed { 1 } else { 0 }
    )
}
// Zero Page - Like absolute, but uses only 1 byte for address & uses 0x00 for the high byte of the address
fn zero_page(cpu: &CPU) -> (OpcodeData, usize) {
    let zpage_address = cpu.read(cpu.get_pc() + 1) as u16;
    let data = cpu.read(zpage_address);

    (
        OpcodeData{ 
            data: Some(data), 
            address: Some(zpage_address), 
            offset: None 
        },
        0
    )
}
// Indexed Addressing Zero-Page (X) - Like Indexed x, but uses only the single next byte as the
// low order byte of the absolute address and fills the top byte w/ 0x00. Then adds x to get
// the effective address. Note that the effective address will never go off the zero-page, if
// the address exceeds 0x00FF, it will loop back around to 0x0000.
// Also known as Zero Page X addressing.
fn zpage_x(cpu: &CPU) -> (OpcodeData, usize) {
    let zpage_address = cpu.read(cpu.get_pc() + 1);
    let effective_zpage_address = zpage_address.wrapping_add(cpu.get_x_reg()) as u16;
    let data = cpu.read(effective_zpage_address);

    (
        OpcodeData{ 
            data: Some(data), 
            address: Some(effective_zpage_address), 
            offset: None 
        },
        0
    )
}
// Indexed Addressing Zero-Page (Y) - Like Indexed Z-Page x, but uses the y register instead
// Also known as Zero Page Y addressing.
fn zpage_y(cpu: &CPU) -> (OpcodeData, usize) {
    let zpage_address = cpu.read(cpu.get_pc() + 1);
    let effective_zpage_address = zpage_address.wrapping_add(cpu.get_y_reg()) as u16;
    let data = cpu.read(effective_zpage_address);

    (
        OpcodeData{ 
            data: Some(data), 
            address: Some(effective_zpage_address), 
            offset: None 
        },
        0
    )
}
// Indirect Addressing - Uses the next 2 bytes as the abs address, then reads the byte that
// points to in memory and the one after (in LLHH order) and uses those as the effective
// address where the data will be read from.
// Note: This mode has a hardware bug where a page boundary cannot be crossed by 
// the reading of 2 bytes from abs_address, and therefore it can take no
// additional clock cycles.
fn indirect(cpu: &CPU) -> (OpcodeData, usize) {
    let abs_address = cpu.read_word(cpu.get_pc() + 1);

    let effective_lo = cpu.read(abs_address) as u16;
    let effective_hi = if abs_address & 0xFF == 0xFF {
        cpu.read(abs_address & 0xFF00)
    } else {
        cpu.read(abs_address + 1)
    } as u16;

    let effective_address = (effective_hi << 8) | effective_lo;
    let data = cpu.read(effective_address);

    (
        OpcodeData{ 
            data: Some(data), 
            address: Some(effective_address), 
            offset: None 
        },
        0
    )
}
// Pre-Indexed Indirect Zero-Page (X) - Like Indexed Z-Page x, but as in Indirect addressing, another
// address is read from memory instead of the data itcpu. The address read is the effective
// address of the actual data. Note that if the z-page address is 0x00FF, then the bytes at
// 0x00FF and 0x0000 are read and used as low and high bytes of the effective address, respectively
fn indirect_x(cpu: &CPU) -> (OpcodeData, usize) {
    let zpage_address = cpu.read(cpu.get_pc() + 1).wrapping_add(cpu.get_x_reg());
    let effective_address = cpu.read_zpage_word(zpage_address);
    let data = cpu.read(effective_address);

    (
        OpcodeData{ 
            data: Some(data), 
            address: Some(effective_address), 
            offset: None 
        },
        0
    )
}
// Post-Indexed Indirect Zero-Page (Y) - Like Indirect Indexed Z-Page x, but with two major differences:
// First, the y register is used instead of x. Second, the register is added to the address
// retrieved from the z-page, not the address used to access the z-page.
fn indirect_y(cpu: &CPU) -> (OpcodeData, usize) {
    let zpage_address = cpu.read(cpu.get_pc() + 1);
    let abs_address = cpu.read_zpage_word(zpage_address);
    let effective_address = abs_address + cpu.get_y_reg() as u16;
    let data = cpu.read(effective_address);

    let page_boundary_crossed = (abs_address & 0xFF00) != (effective_address & 0xFF00);

    (
        OpcodeData{ 
            data: Some(data), 
            address: Some(effective_address), 
            offset: None 
        },
        if page_boundary_crossed { 1 } else { 0 }
    )
}
// Relative Addressing - Gets the next bytes as a signed byte to be added to the pc for branch
// instructions.
fn relative(cpu: &CPU) -> (OpcodeData, usize) {
    let data = cpu.read(cpu.get_pc() + 1) as i8;
    let address = (cpu.get_pc() as i32 + data as i32) as u16;

    (
        OpcodeData{ 
            data: None, 
            address: Some(address), 
            offset: Some(data) 
        },
        0
    )
}

// OPCODES - all the cpu instructions
// Args:
//  - data: Single byte of data read from memory. Where data comes from is determined by the
//          addressing mode of the instruction.
//  - address: Some instructions also need to write to memory, so the address that data was
//             taken from can also be included as an argument.
//
// Returns:
//  - Number of clock cycles taken by the instruction

// ADC - Add Memory to Accumulator with Carry
fn adc(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    let data = opcode_data.data.unwrap();

    let result = (data as u16) + (cpu.get_acc() as u16) + (cpu.get_carry_flag() as u16);
    cpu.set_carry_flag(if result & 0xFF00 > 0 { 1 } else { 0 });
    cpu.set_zero_flag(if result & 0xFF == 0 { 1 } else { 0 });
    cpu.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
    
    // Set V flag if acc and data are same sign, but result is different sign
    let a = cpu.get_acc() & 0x80 != 0;
    let r = (result & 0x80) != 0;
    let d = data & 0x80 != 0;
    cpu.set_overflow_flag(if !(a^d)&(a^r) { 1 } else { 0 });

    cpu.set_acc(result as u8);

    0
}
// AND - AND Memory with Accumulator
fn and(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    let data = opcode_data.data.unwrap();

    let result = cpu.get_acc() & data;
    cpu.set_zero_flag(if result == 0 { 1 } else { 0 });
    cpu.set_negative_flag(if result & 0x80 > 0 { 1 } else { 0 });
    0
}
// ASL - Shift Left One Bit (Memory or Accumulator)
fn asl(_: &mut CPU, _: OpcodeData) -> usize {
    // skipping for now
    0
}
// BCC - Branch on Carry Clear
fn bcc(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    let offset = opcode_data.offset.unwrap();

    if cpu.get_carry_flag() != 1 {
        cpu.set_pc((cpu.get_pc() as i32 + offset as i32) as u16);
    }
    0
}
// BCS - Branch on Carry Set
fn bcs(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    let offset = opcode_data.offset.unwrap();

    if cpu.get_carry_flag() == 1 {
        cpu.set_pc((cpu.get_pc() as i32 + offset as i32) as u16);
    }
    0
}
// BEQ - Branch on Equal (Zero flag set)
fn beq(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    let offset = opcode_data.offset.unwrap();

    if cpu.get_zero_flag() == 1 {
        cpu.set_pc((cpu.get_pc() as i32 + offset as i32) as u16);
    }
    
    0
}
// BIT - Test Bits in Memory with Accumulator
fn bit(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    let data = opcode_data.data.unwrap();

    cpu.set_negative_flag(if data & 0x80 != 0 { 1 } else { 0 });
    cpu.set_overflow_flag(if data & 0x40 > 0 { 1 } else { 0 });
    cpu.set_zero_flag(if data & cpu.get_acc() == 0 { 1 } else { 0 });
    0
}
// BMI - Branch on Result Minus (Negative flag set)
fn bmi(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    let offset = opcode_data.offset.unwrap();

    if cpu.get_negative_flag() == 1 {
        cpu.set_pc((cpu.get_pc() as i32 + offset as i32) as u16);
    }
    0
}
// BNE - Branch on Not Equal (Zero flag NOT set)
fn bne(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    let offset = opcode_data.offset.unwrap();

    if cpu.get_zero_flag() == 0 {
        cpu.set_pc((cpu.get_pc() as i32 + offset as i32) as u16);
    }
    0
}
// BPL - Branch on Result Plus (Negative flag NOT set)
fn bpl(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    let offset = opcode_data.offset.unwrap();

    if cpu.get_negative_flag() == 0 {
        cpu.set_pc((cpu.get_pc() as i32 + offset as i32) as u16);
    }
    0
}
// BRK - Force Break (Initiate interrupt)
fn brk(cpu: &mut CPU, _: OpcodeData) -> usize {
    cpu.irq();

    0
}
// BVC - Branch on Overflow clear
fn bvc(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    let offset = opcode_data.offset.unwrap();

    if cpu.get_overflow_flag() == 0 {
        cpu.set_pc((cpu.get_pc() as i32 + offset as i32) as u16);
    }
    0
}
// BVS - Branch on Overflow set
fn bvs(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    let offset = opcode_data.offset.unwrap();

    if cpu.get_overflow_flag() == 1 {
        cpu.set_pc((cpu.get_pc() as i32 + offset as i32) as u16);
    }
    0
}
// CLC - Clear Carry Flag
fn clc(cpu: &mut CPU, _: OpcodeData) -> usize {
    cpu.set_carry_flag(0);
    0
}
// CLD - Clear Decimal Mode
fn cld(cpu: &mut CPU, _: OpcodeData) -> usize {
    cpu.set_decimal_flag(0);
    0
}
// CLI - Clear Interrupt Disable Bit
fn cli(cpu: &mut CPU, _: OpcodeData) -> usize {
    cpu.set_interrupt_flag(0);
    0
}
// CLV - Clear Overflow Flag
fn clv(cpu: &mut CPU, _: OpcodeData) -> usize {
    cpu.set_overflow_flag(0);
    0
}
// CMP - Compare Memory with Accumulator
fn cmp(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    let data = opcode_data.data.unwrap();

    let result = (cpu.get_acc() as i16) - (data as i16);
    cpu.set_zero_flag(if result == 0 { 1 } else { 0 });
    cpu.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
    cpu.set_carry_flag(if result >= 0 { 1 } else { 0 });
    0
}
// CPX - Compare Memory and Index X
fn cpx(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    let data = opcode_data.data.unwrap();

    let result = (cpu.get_x_reg() as i16) - (data as i16);
    cpu.set_zero_flag(if result == 0 { 1 } else { 0 });
    cpu.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
    cpu.set_carry_flag(if result >= 0 { 1 } else { 0 });
    0
}
// CPY - Compare Memory and Index Y
fn cpy(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    let data = opcode_data.data.unwrap();

    let result = (cpu.get_y_reg() as i16) - (data as i16);
    cpu.set_zero_flag(if result == 0 { 1 } else { 0 });
    cpu.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
    cpu.set_carry_flag(if result >= 0 { 1 } else { 0 });
    0
}
// DEC - Decrement Memory
fn dec(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    let data = opcode_data.data.unwrap();
    let address = opcode_data.address.unwrap();

    let result = data.wrapping_sub(1);
    cpu.set_zero_flag(if result == 0 { 1 } else { 0 });
    cpu.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
    cpu.write(address, result);
    0
}
// DEX - Decrement X Register
fn dex(cpu: &mut CPU, _: OpcodeData) -> usize {
    let result = cpu.get_x_reg().wrapping_sub(1);
    cpu.set_zero_flag(if result == 0 { 1 } else { 0 });
    cpu.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
    cpu.set_x_reg(result);
    0
}
// DEY - Decrement Y Register
fn dey(cpu: &mut CPU, _: OpcodeData) -> usize {
    let result = cpu.get_y_reg().wrapping_sub(1);
    cpu.set_zero_flag(if result == 0 { 1 } else { 0 });
    cpu.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
    cpu.set_y_reg(result);
    0
}
// EOR - Exclusive OR
fn eor(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    let data = opcode_data.data.unwrap();

    let result = cpu.get_acc() ^ data;
    cpu.set_zero_flag(if result == 0 { 1 } else { 0 });
    cpu.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
    cpu.set_acc(result);
    0
}
// INC - Increment Memory
fn inc(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    let data = opcode_data.data.unwrap();
    let address = opcode_data.address.unwrap();

    let result = data.wrapping_add(1);
    cpu.set_zero_flag(if result == 0 { 1 } else { 0 });
    cpu.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
    cpu.write(address, result);
    0
}
// INX - Increment X Register
fn inx(cpu: &mut CPU, _: OpcodeData) -> usize {
    let result = cpu.get_x_reg().wrapping_add(1);
    cpu.set_zero_flag(if result == 0 { 1 } else { 0 });
    cpu.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
    cpu.set_x_reg(result);
    0
}
// INY - Increment Y Register
fn iny(cpu: &mut CPU, _: OpcodeData) -> usize {
    let result = cpu.get_y_reg().wrapping_add(1);
    cpu.set_zero_flag(if result == 0 { 1 } else { 0 });
    cpu.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
    cpu.set_y_reg(result);
    0
}
// JMP - Jump
fn jmp(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    let address = opcode_data.address.unwrap();

    cpu.set_pc(address);
    0
}
// JSR - Jump to Subroutine
fn jsr(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    let address = opcode_data.address.unwrap();

    let return_point = cpu.get_pc() - 1; // Return point is the return address - 1, and since this
    let lo = return_point as u8;
    let hi = (return_point >> 8) as u8;

    cpu.push_to_stack(lo);
    cpu.push_to_stack(hi);
    cpu.set_pc(address);

    0
}
// LDA - Load Accumulator
fn lda(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    let data = opcode_data.data.unwrap();

    cpu.set_zero_flag(if data == 0 { 1 } else { 0 });
    cpu.set_negative_flag(if data & 0x80 != 0 { 1 } else { 0 });
    cpu.set_acc(data);
    0
}
// LDX - Load X Register
fn ldx(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    let data = opcode_data.data.unwrap();

    cpu.set_zero_flag(if data == 0 { 1 } else { 0 });
    cpu.set_negative_flag(if data & 0x80 != 0 { 1 } else { 0 });
    cpu.set_x_reg(data);
    0
}
// LDY - Load Y Register
fn ldy(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    let data = opcode_data.data.unwrap();

    cpu.set_zero_flag(if data == 0 { 1 } else { 0 });
    cpu.set_negative_flag(if data & 0x80 != 0 { 1 } else { 0 });
    cpu.set_y_reg(data);
    0
}
// LSR - Logical Shift Right
fn lsr(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    // Memory version if data & address given
    if let (Some(data), Some(address)) = (opcode_data.data, opcode_data.address) {
        let result = data >> 1;
        cpu.set_carry_flag(data & 0x01);
        cpu.set_zero_flag(if result == 0 { 1 } else { 0 });
        cpu.set_negative_flag(0); // result will always have bit 7 == 0
        cpu.write(address, result);
    }
    // Otherwise, Accumulator version
    else {
        let result = cpu.get_acc() >> 1;
        cpu.set_carry_flag(cpu.get_acc() & 0x01);
        cpu.set_zero_flag(if result == 0 { 1 } else { 0 });
        cpu.set_negative_flag(0); // result will always have bit 7 == 0
        cpu.set_acc(result);
    }

    0
}
// NOP - No Operation
fn nop(_: &mut CPU, _: OpcodeData) -> usize { 1 }
// ORA - Logical Inclusive OR
fn ora(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    let data = opcode_data.data.unwrap();

    let result = cpu.get_acc() | data;
    cpu.set_zero_flag(if result == 0 { 1 } else { 0 });
    cpu.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
    cpu.set_acc(result);
    0
}
// PHA - Push Accumulator
fn pha(cpu: &mut CPU, _: OpcodeData) -> usize {
    cpu.push_to_stack(cpu.get_acc());
    0
}
// PHP - Push Processor Status
fn php(cpu: &mut CPU, _: OpcodeData) -> usize {
    cpu.push_to_stack(cpu.get_flags());
    0
}
// PLA - Pull Accumulator
fn pla(cpu: &mut CPU, _: OpcodeData) -> usize {
    let result = cpu.pop_from_stack();
    cpu.set_zero_flag(if result == 0 { 1 } else { 0 });
    cpu.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
    cpu.set_acc(result);
    0
}
// PLP - Pull Processor Status
fn plp(cpu: &mut CPU, _: OpcodeData) -> usize {
    let data = cpu.pop_from_stack();
    cpu.set_flags(data);
    0
}
// ROL - Rotate Left
fn rol(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    // Memory Version if data & address given
    if let (Some(data), Some(address)) = (opcode_data.data, opcode_data.address) {
        let result = (data << 1) | cpu.get_carry_flag();
        cpu.set_carry_flag(data >> 7); // old bit 7 becomes new carry
        cpu.set_zero_flag(if result == 0 { 1 } else { 0 });
        cpu.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
        cpu.write(address, result);
    }
    // Otherwise, Accumulator version
    else {
        let result = (cpu.get_acc() << 1) | cpu.get_carry_flag();
        cpu.set_carry_flag(cpu.get_acc() >> 7); // old bit 7 becomes new carry
        cpu.set_zero_flag(if result == 0 { 1 } else { 0 });
        cpu.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
        cpu.set_acc(result);
    }

    0
}
// ROR - Rotate Right
fn ror(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    // Memory version if data & address given
    if let (Some(data), Some(address)) = (opcode_data.data, opcode_data.address) {
        let result = (cpu.get_carry_flag() << 7) | (data >> 1);
        cpu.set_carry_flag(data & 0x01); // old bit 0 becomes new carry
        cpu.set_zero_flag(if result == 0 { 1 } else { 0 });
        cpu.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
        cpu.write(address, result);
    } 
    // Otherwise, Accumulator Version
    else {
        let result = (cpu.get_carry_flag() << 7) | (cpu.get_acc() >> 1);
        cpu.set_carry_flag(cpu.get_acc() & 0x01); // old bit 0 becomes new carry
        cpu.set_zero_flag(if result == 0 { 1 } else { 0 });
        cpu.set_negative_flag(if result & 0x80 != 0 { 1 } else { 0 });
        cpu.set_acc(result);
    }

    0
}
// RTI - Return from Interrupt
fn rti(cpu: &mut CPU, _: OpcodeData) -> usize {
    // Restore processer status
    let prev_status = cpu.pop_from_stack();
    cpu.set_flags(prev_status);
    cpu.set_b_flag(0);
    cpu.set_unused_flag(0);
    // Return to previous PC
    let hi = cpu.pop_from_stack() as u16;
    let lo = cpu.pop_from_stack() as u16;
    cpu.set_pc((hi << 8) | lo);

    0
}
// RTS - Return from Subroutine
fn rts(cpu: &mut CPU, _: OpcodeData) -> usize {
    let hi = cpu.pop_from_stack() as u16;
    let lo = cpu.pop_from_stack() as u16;
    cpu.set_pc((hi << 8) | lo);
    0
}
// SBC - Subtract with Carry
fn sbc(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    let data = opcode_data.data.unwrap();
    let twos_comp = (data as i8 * -1) as u8;

    let new_opcode_data = OpcodeData{
        data: Some(twos_comp), 
        address: opcode_data.address, 
        offset: opcode_data.offset
    }; 

    adc(cpu, new_opcode_data)
}
// SEC - Set Carry Flag
fn sec(cpu: &mut CPU, _: OpcodeData) -> usize {
    cpu.set_carry_flag(1);
    0
}
// SED - Set Decimal Flag
fn sed(cpu: &mut CPU, _: OpcodeData) -> usize {
    cpu.set_decimal_flag(1);
    0
}
// SEI - Set Interrupt Disable
fn sei(cpu: &mut CPU, _: OpcodeData) -> usize {
    cpu.set_interrupt_flag(1);
    0
}
// STA - Store Accumulator
fn sta(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    let address = opcode_data.address.unwrap();
    cpu.write(address, cpu.get_acc());
    0
}
// STX - Store X Register
fn stx(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    let address = opcode_data.address.unwrap();
    cpu.write(address, cpu.get_x_reg());
    0
}
// STY - Store Y Register
fn sty(cpu: &mut CPU, opcode_data: OpcodeData) -> usize {
    let address = opcode_data.address.unwrap();
    cpu.write(address, cpu.get_y_reg());
    0
}
// TAX - Transfer Accumulator to X
fn tax(cpu: &mut CPU, _: OpcodeData) -> usize {
    cpu.set_x_reg(cpu.get_acc());
    cpu.set_zero_flag(if cpu.get_x_reg() == 0 { 1 } else { 0 });
    cpu.set_negative_flag(if cpu.get_x_reg() & 0x80 != 0 { 1 } else { 0 });
    0
}
// TAY - Transfer Accumulator to Y
fn tay(cpu: &mut CPU, _: OpcodeData) -> usize {
    cpu.set_y_reg(cpu.get_acc());
    cpu.set_zero_flag(if cpu.get_y_reg() == 0 { 1 } else { 0 });
    cpu.set_negative_flag(if cpu.get_y_reg() & 0x80 != 0 { 1 } else { 0 });
    0
}
// TSX - Transfer Stack Pointer to X
fn tsx(cpu: &mut CPU, _: OpcodeData) -> usize {
    cpu.set_x_reg(cpu.get_sp());
    cpu.set_zero_flag(if cpu.get_x_reg() == 0 { 1 } else { 0 });
    cpu.set_negative_flag(if cpu.get_x_reg() & 0x80 != 0 { 1 } else { 0 });
    0
}
// TXA - Transfer X to Accumulator
fn txa(cpu: &mut CPU, _: OpcodeData) -> usize {
    cpu.set_acc(cpu.get_x_reg());
    cpu.set_zero_flag(if cpu.get_acc() == 0 { 1 } else { 0 });
    cpu.set_negative_flag(if cpu.get_acc() & 0x80 != 0 { 1 } else { 0 });
    0
}
// TXS - Transfer X to Stack Pointer
fn txs(cpu: &mut CPU, _: OpcodeData) -> usize {
    cpu.set_sp(cpu.get_x_reg());
    0
}
// TYA - Transfer Y to Accumulator
fn tya(cpu: &mut CPU, _: OpcodeData) -> usize {
    cpu.set_acc(cpu.get_y_reg());
    cpu.set_zero_flag(if cpu.get_acc() == 0 { 1 } else { 0 });
    cpu.set_negative_flag(if cpu.get_acc() & 0x80 != 0 { 1 } else { 0 });
    0
}

// INVALID OPCODE - An unimplemented opcode not recognized by the CPU
fn xxx(_: &mut CPU, _: OpcodeData) -> usize { 0 }