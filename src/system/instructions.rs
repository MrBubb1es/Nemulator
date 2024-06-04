// KNOWN ISSUES:
//  - Not all instructions that use abs_x, abs_y, etc. should add the extra 
//    clock cycle if page boundary crossed. (e.g. SBC does but STA does not)

use crate::system::cpu::Cpu6502;

// Generic unimplemented opcode instr with given opcode (for differentiating instr calls)
macro_rules! illegal_op {
    ($opcode:expr) => {
        Instruction {
            name: "???",
            opcode_num: $opcode,
            addr_mode: AddressingMode::Implied,
            addr_func: |_| { (0, 0) },
            func: xxx,
            base_clocks: 2,
            bytes: 1,
            has_extra_fetch_cycles: false, 
            is_illegal: true,
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
    Instruction{name: "BRK", opcode_num: 0x00, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: brk, base_clocks: 7, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ORA", opcode_num: 0x01, addr_mode: AddressingMode::IndirectX, addr_func: indirect_x, func: ora, base_clocks: 6, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    illegal_op!(0x02), // JAM
    Instruction{name: "SLO", opcode_num: 0x03, addr_mode: AddressingMode::IndirectX, addr_func: indirect_x, func: slo, base_clocks: 8, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "NOP", opcode_num: 0x04, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: nop, base_clocks: 3, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "ORA", opcode_num: 0x05, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: ora, base_clocks: 3, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ASL", opcode_num: 0x06, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: asl_mem, base_clocks: 5, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "SLO", opcode_num: 0x07, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: slo, base_clocks: 5, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "PHP", opcode_num: 0x08, addr_mode: AddressingMode::Implied, addr_func: implied, func: php, base_clocks: 3, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ORA", opcode_num: 0x09, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: ora, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ASL", opcode_num: 0x0A, addr_mode: AddressingMode::Accumulator, addr_func: accumulator, func: asl_acc, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ANC", opcode_num: 0x0B, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: anc, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "NOP", opcode_num: 0x0C, addr_mode: AddressingMode::Absolute, addr_func: zpage_x, func: nop, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "ORA", opcode_num: 0x0D, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: ora, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ASL", opcode_num: 0x0E, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: asl_mem, base_clocks: 6, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "SLO", opcode_num: 0x0F, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: slo, base_clocks: 6, bytes: 3, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "BPL", opcode_num: 0x10, addr_mode: AddressingMode::Relative, addr_func: relative, func: bpl, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: true, is_illegal: false},
    Instruction{name: "ORA", opcode_num: 0x11, addr_mode: AddressingMode::IndirectY, addr_func: indirect_y, func: ora, base_clocks: 5, bytes: 2, has_extra_fetch_cycles: true, is_illegal: false},
    illegal_op!(0x12), // JAM
    Instruction{name: "SLO", opcode_num: 0x13, addr_mode: AddressingMode::IndirectY, addr_func: indirect_y, func: slo, base_clocks: 8, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "NOP", opcode_num: 0x14, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: nop, base_clocks: 4, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "ORA", opcode_num: 0x15, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: ora, base_clocks: 4, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ASL", opcode_num: 0x16, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: asl_mem, base_clocks: 6, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "SLO", opcode_num: 0x17, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: slo, base_clocks: 6, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "CLC", opcode_num: 0x18, addr_mode: AddressingMode::Implied, addr_func: implied, func: clc, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ORA", opcode_num: 0x19, addr_mode: AddressingMode::AbsoluteY, addr_func: absolute_y, func: ora, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: true, is_illegal: false},
    Instruction{name: "NOP", opcode_num: 0x1A, addr_mode: AddressingMode::Implied, addr_func: implied, func: nop, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "SLO", opcode_num: 0x1B, addr_mode: AddressingMode::AbsoluteY, addr_func: absolute_y, func: slo, base_clocks: 7, bytes: 3, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "NOP", opcode_num: 0x1C, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: nop, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: true, is_illegal: true},
    Instruction{name: "ORA", opcode_num: 0x1D, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: ora, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: true, is_illegal: false},
    Instruction{name: "ASL", opcode_num: 0x1E, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: asl_mem, base_clocks: 7, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "SLO", opcode_num: 0x1F, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: slo, base_clocks: 7, bytes: 3, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "JSR", opcode_num: 0x20, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: jsr, base_clocks: 6, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "AND", opcode_num: 0x21, addr_mode: AddressingMode::IndirectX, addr_func: indirect_x, func: and, base_clocks: 6, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    illegal_op!(0x22), // JAM
    Instruction{name: "RLA", opcode_num: 0x23, addr_mode: AddressingMode::IndirectX, addr_func: indirect_x, func: rla, base_clocks: 8, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "BIT", opcode_num: 0x24, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: bit, base_clocks: 3, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "AND", opcode_num: 0x25, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: and, base_clocks: 3, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ROL", opcode_num: 0x26, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: rol_mem, base_clocks: 5, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "RLA", opcode_num: 0x27, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: rla, base_clocks: 5, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "PLP", opcode_num: 0x28, addr_mode: AddressingMode::Implied, addr_func: implied, func: plp, base_clocks: 4, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "AND", opcode_num: 0x29, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: and, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ROL", opcode_num: 0x2A, addr_mode: AddressingMode::Accumulator, addr_func: accumulator, func: rol_acc, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ANC", opcode_num: 0x2B, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: anc, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "BIT", opcode_num: 0x2C, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: bit, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "AND", opcode_num: 0x2D, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: and, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ROL", opcode_num: 0x2E, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: rol_mem, base_clocks: 6, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "RLA", opcode_num: 0x2F, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: rla, base_clocks: 6, bytes: 3, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "BMI", opcode_num: 0x30, addr_mode: AddressingMode::Relative, addr_func: relative, func: bmi, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: true, is_illegal: false},
    Instruction{name: "AND", opcode_num: 0x31, addr_mode: AddressingMode::IndirectY, addr_func: indirect_y, func: and, base_clocks: 5, bytes: 2, has_extra_fetch_cycles: true, is_illegal: false},
    illegal_op!(0x32), // JAM
    Instruction{name: "RLA", opcode_num: 0x2F, addr_mode: AddressingMode::IndirectY, addr_func: indirect_y, func: rla, base_clocks: 8, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "NOP", opcode_num: 0x34, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: nop, base_clocks: 4, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "AND", opcode_num: 0x35, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: and, base_clocks: 4, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ROL", opcode_num: 0x36, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: rol_mem, base_clocks: 6, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "RLA", opcode_num: 0x2F, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: rla, base_clocks: 6, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "SEC", opcode_num: 0x38, addr_mode: AddressingMode::Implied, addr_func: implied, func: sec, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "AND", opcode_num: 0x39, addr_mode: AddressingMode::AbsoluteY, addr_func: absolute_y, func: and, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: true, is_illegal: false},
    Instruction{name: "NOP", opcode_num: 0x3A, addr_mode: AddressingMode::Implied, addr_func: implied, func: nop, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "RLA", opcode_num: 0x3B, addr_mode: AddressingMode::AbsoluteY, addr_func: absolute_y, func: rla, base_clocks: 7, bytes: 3, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "NOP", opcode_num: 0x3C, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: nop, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: true, is_illegal: true},
    Instruction{name: "AND", opcode_num: 0x3D, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: and, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: true, is_illegal: false},
    Instruction{name: "ROL", opcode_num: 0x3E, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: rol_mem, base_clocks: 7, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "RLA", opcode_num: 0x3F, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: rla, base_clocks: 7, bytes: 3, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "RTI", opcode_num: 0x40, addr_mode: AddressingMode::Implied, addr_func: implied, func: rti, base_clocks: 6, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "EOR", opcode_num: 0x41, addr_mode: AddressingMode::IndirectX, addr_func: indirect_x, func: eor, base_clocks: 6, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    illegal_op!(0x42), // JAM
    Instruction{name: "SRE", opcode_num: 0x43, addr_mode: AddressingMode::IndirectX, addr_func: indirect_x, func: sre, base_clocks: 8, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "NOP", opcode_num: 0x44, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: nop, base_clocks: 3, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "EOR", opcode_num: 0x45, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: eor, base_clocks: 3, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "LSR", opcode_num: 0x46, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: lsr_mem, base_clocks: 5, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "SRE", opcode_num: 0x47, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: sre, base_clocks: 5, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "PHA", opcode_num: 0x48, addr_mode: AddressingMode::Implied, addr_func: implied, func: pha, base_clocks: 3, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "EOR", opcode_num: 0x49, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: eor, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "LSR", opcode_num: 0x4A, addr_mode: AddressingMode::Accumulator, addr_func: accumulator, func: lsr_acc, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ASR", opcode_num: 0x4B, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: asr, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "JMP", opcode_num: 0x4C, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: jmp, base_clocks: 3, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "EOR", opcode_num: 0x4D, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: eor, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "LSR", opcode_num: 0x4E, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: lsr_mem, base_clocks: 6, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "SRE", opcode_num: 0x4F, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: sre, base_clocks: 6, bytes: 3, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "BVC", opcode_num: 0x50, addr_mode: AddressingMode::Relative, addr_func: relative, func: bvc, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: true, is_illegal: false},
    Instruction{name: "EOR", opcode_num: 0x51, addr_mode: AddressingMode::IndirectY, addr_func: indirect_y, func: eor, base_clocks: 5, bytes: 2, has_extra_fetch_cycles: true, is_illegal: false},
    illegal_op!(0x52), // JAM
    Instruction{name: "SRE", opcode_num: 0x4F, addr_mode: AddressingMode::IndirectY, addr_func: indirect_y, func: sre, base_clocks: 8, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "NOP", opcode_num: 0x54, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: nop, base_clocks: 4, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "EOR", opcode_num: 0x55, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: eor, base_clocks: 4, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "LSR", opcode_num: 0x56, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: lsr_mem, base_clocks: 6, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "SRE", opcode_num: 0x57, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: sre, base_clocks: 6, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "CLI", opcode_num: 0x58, addr_mode: AddressingMode::Implied, addr_func: implied, func: cli, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "EOR", opcode_num: 0x59, addr_mode: AddressingMode::AbsoluteY, addr_func: absolute_y, func: eor, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: true, is_illegal: false},
    Instruction{name: "NOP", opcode_num: 0x5A, addr_mode: AddressingMode::Implied, addr_func: implied, func: nop, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "SRE", opcode_num: 0x5B, addr_mode: AddressingMode::AbsoluteY, addr_func: absolute_y, func: sre, base_clocks: 7, bytes: 3, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "NOP", opcode_num: 0x5C, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: nop, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: true, is_illegal: true},
    Instruction{name: "EOR", opcode_num: 0x5D, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: eor, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: true, is_illegal: false},
    Instruction{name: "LSR", opcode_num: 0x5E, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: lsr_mem, base_clocks: 7, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "SRE", opcode_num: 0x5F, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: sre, base_clocks: 7, bytes: 3, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "RTS", opcode_num: 0x60, addr_mode: AddressingMode::Implied, addr_func: implied, func: rts, base_clocks: 6, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ADC", opcode_num: 0x61, addr_mode: AddressingMode::IndirectX, addr_func: indirect_x, func: adc, base_clocks: 6, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    illegal_op!(0x62), // JAM
    Instruction{name: "RRA", opcode_num: 0x63, addr_mode: AddressingMode::IndirectX, addr_func: indirect_x, func: rra, base_clocks: 8, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "NOP", opcode_num: 0x64, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: nop, base_clocks: 3, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "ADC", opcode_num: 0x65, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: adc, base_clocks: 3, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ROR", opcode_num: 0x66, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: ror_mem, base_clocks: 5, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "RRA", opcode_num: 0x67, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: rra, base_clocks: 5, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "PLA", opcode_num: 0x68, addr_mode: AddressingMode::Implied, addr_func: implied, func: pla, base_clocks: 4, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ADC", opcode_num: 0x69, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: adc, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ROR", opcode_num: 0x6A, addr_mode: AddressingMode::Accumulator, addr_func: accumulator, func: ror_acc, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ARR", opcode_num: 0x6B, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: arr, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "JMP", opcode_num: 0x6C, addr_mode: AddressingMode::Indirect, addr_func: indirect, func: jmp, base_clocks: 5, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ADC", opcode_num: 0x6D, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: adc, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ROR", opcode_num: 0x6E, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: ror_mem, base_clocks: 6, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "RRA", opcode_num: 0x6F, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: rra, base_clocks: 6, bytes: 3, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "BVS", opcode_num: 0x70, addr_mode: AddressingMode::Relative, addr_func: relative, func: bvs, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: true, is_illegal: false},
    Instruction{name: "ADC", opcode_num: 0x71, addr_mode: AddressingMode::IndirectY, addr_func: indirect_y, func: adc, base_clocks: 5, bytes: 2, has_extra_fetch_cycles: true, is_illegal: false},
    illegal_op!(0x72), // JAM
    Instruction{name: "RRA", opcode_num: 0x73, addr_mode: AddressingMode::IndirectY, addr_func: indirect_y, func: rra, base_clocks: 8, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "NOP", opcode_num: 0x74, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: nop, base_clocks: 4, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "ADC", opcode_num: 0x75, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: adc, base_clocks: 4, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ROR", opcode_num: 0x76, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: ror_mem, base_clocks: 6, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "RRA", opcode_num: 0x77, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: rra, base_clocks: 6, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "SEI", opcode_num: 0x78, addr_mode: AddressingMode::Implied, addr_func: implied, func: sei, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ADC", opcode_num: 0x79, addr_mode: AddressingMode::AbsoluteY, addr_func: absolute_y, func: adc, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: true, is_illegal: false},
    Instruction{name: "NOP", opcode_num: 0x7A, addr_mode: AddressingMode::Implied, addr_func: implied, func: nop, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "RRA", opcode_num: 0x7B, addr_mode: AddressingMode::AbsoluteY, addr_func: absolute_y, func: rra, base_clocks: 7, bytes: 3, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "NOP", opcode_num: 0x7C, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: nop, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: true, is_illegal: true},
    Instruction{name: "ADC", opcode_num: 0x7D, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: adc, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: true, is_illegal: false},
    Instruction{name: "ROR", opcode_num: 0x7E, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: ror_mem, base_clocks: 7, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "RRA", opcode_num: 0x7F, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: rra, base_clocks: 7, bytes: 3, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "NOP", opcode_num: 0x80, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: nop, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "STA", opcode_num: 0x81, addr_mode: AddressingMode::IndirectX, addr_func: indirect_x, func: sta, base_clocks: 6, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "NOP", opcode_num: 0x82, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: nop, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "SAX", opcode_num: 0x83, addr_mode: AddressingMode::IndirectX, addr_func: indirect_x, func: sax, base_clocks: 6, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "STY", opcode_num: 0x84, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: sty, base_clocks: 3, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "STA", opcode_num: 0x85, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: sta, base_clocks: 3, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "STX", opcode_num: 0x86, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: stx, base_clocks: 3, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "SAX", opcode_num: 0x87, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: sax, base_clocks: 3, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "DEY", opcode_num: 0x88, addr_mode: AddressingMode::Implied, addr_func: implied, func: dey, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "NOP", opcode_num: 0x89, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: nop, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "TXA", opcode_num: 0x8A, addr_mode: AddressingMode::Implied, addr_func: implied, func: txa, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    illegal_op!(0x8B), // XAA - non-deterministic, highly unstable, do not use
    Instruction{name: "STY", opcode_num: 0x8C, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: sty, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "STA", opcode_num: 0x8D, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: sta, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "STX", opcode_num: 0x8E, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: stx, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "SAX", opcode_num: 0x8F, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: sax, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "BCC", opcode_num: 0x90, addr_mode: AddressingMode::Relative, addr_func: relative, func: bcc, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: true, is_illegal: false},
    Instruction{name: "STA", opcode_num: 0x91, addr_mode: AddressingMode::IndirectY, addr_func: indirect_y, func: sta, base_clocks: 6, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    illegal_op!(0x92), // JAM
    illegal_op!(0x93), // SHA
    Instruction{name: "STY", opcode_num: 0x94, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: sty, base_clocks: 4, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "STA", opcode_num: 0x95, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: sta, base_clocks: 4, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "STX", opcode_num: 0x96, addr_mode: AddressingMode::ZeroPageY, addr_func: zpage_y, func: stx, base_clocks: 4, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "SAX", opcode_num: 0x97, addr_mode: AddressingMode::ZeroPageY, addr_func: zpage_y, func: sax, base_clocks: 4, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "TYA", opcode_num: 0x98, addr_mode: AddressingMode::Implied, addr_func: implied, func: tya, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "STA", opcode_num: 0x99, addr_mode: AddressingMode::AbsoluteY, addr_func: absolute_y, func: sta, base_clocks: 5, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "TXS", opcode_num: 0x9A, addr_mode: AddressingMode::Implied, addr_func: implied, func: txs, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    illegal_op!(0x9B), // SHA - IndirectY
    illegal_op!(0x9C), // SHY
    Instruction{name: "STA", opcode_num: 0x9D, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: sta, base_clocks: 5, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    illegal_op!(0x9E), // SHX
    illegal_op!(0x9F), // SHA - AbsoluteY
    Instruction{name: "LDY", opcode_num: 0xA0, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: ldy, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "LDA", opcode_num: 0xA1, addr_mode: AddressingMode::IndirectX, addr_func: indirect_x, func: lda, base_clocks: 6, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "LDX", opcode_num: 0xA2, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: ldx, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "LAX", opcode_num: 0xA3, addr_mode: AddressingMode::IndirectX, addr_func: indirect_x, func: lax, base_clocks: 6, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "LDY", opcode_num: 0xA4, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: ldy, base_clocks: 3, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "LDA", opcode_num: 0xA5, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: lda, base_clocks: 3, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "LDX", opcode_num: 0xA6, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: ldx, base_clocks: 3, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "LAX", opcode_num: 0xA7, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: lax, base_clocks: 3, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "TAY", opcode_num: 0xA8, addr_mode: AddressingMode::Implied, addr_func: implied, func: tay, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "LDA", opcode_num: 0xA9, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: lda, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "TAX", opcode_num: 0xAA, addr_mode: AddressingMode::Implied, addr_func: implied, func: tax, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    illegal_op!(0xAB), // LXA
    Instruction{name: "LDY", opcode_num: 0xAC, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: ldy, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "LDA", opcode_num: 0xAD, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: lda, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "LDX", opcode_num: 0xAE, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: ldx, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "LAX", opcode_num: 0xAF, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: lax, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "BCS", opcode_num: 0xB0, addr_mode: AddressingMode::Relative, addr_func: relative, func: bcs, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: true, is_illegal: false},
    Instruction{name: "LDA", opcode_num: 0xB1, addr_mode: AddressingMode::IndirectY, addr_func: indirect_y, func: lda, base_clocks: 5, bytes: 2, has_extra_fetch_cycles: true, is_illegal: false},
    illegal_op!(0xB2), // JAM
    Instruction{name: "LAX", opcode_num: 0xB3, addr_mode: AddressingMode::IndirectY, addr_func: indirect_y, func: lax, base_clocks: 5, bytes: 2, has_extra_fetch_cycles: true, is_illegal: true},
    Instruction{name: "LDY", opcode_num: 0xB4, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: ldy, base_clocks: 4, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "LDA", opcode_num: 0xB5, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: lda, base_clocks: 4, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "LDX", opcode_num: 0xB6, addr_mode: AddressingMode::ZeroPageY, addr_func: zpage_y, func: ldx, base_clocks: 4, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "LAX", opcode_num: 0xB7, addr_mode: AddressingMode::ZeroPageY, addr_func: zpage_y, func: lax, base_clocks: 4, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "CLV", opcode_num: 0xB8, addr_mode: AddressingMode::Implied, addr_func: implied, func: clv, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "LDA", opcode_num: 0xB9, addr_mode: AddressingMode::AbsoluteY, addr_func: absolute_y, func: lda, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: true, is_illegal: false},
    Instruction{name: "TSX", opcode_num: 0xBA, addr_mode: AddressingMode::Implied, addr_func: implied, func: tsx, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    illegal_op!(0xBB), // LAS
    Instruction{name: "LDY", opcode_num: 0xBC, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: ldy, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: true, is_illegal: false},
    Instruction{name: "LDA", opcode_num: 0xBD, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: lda, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: true, is_illegal: false},
    Instruction{name: "LDX", opcode_num: 0xBE, addr_mode: AddressingMode::AbsoluteY, addr_func: absolute_y, func: ldx, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: true, is_illegal: false},
    Instruction{name: "LAX", opcode_num: 0xBF, addr_mode: AddressingMode::AbsoluteY, addr_func: absolute_y, func: lax, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: true, is_illegal: true},
    Instruction{name: "CPY", opcode_num: 0xC0, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: cpy, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "CMP", opcode_num: 0xC1, addr_mode: AddressingMode::IndirectX, addr_func: indirect_x, func: cmp, base_clocks: 6, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "NOP", opcode_num: 0xC2, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: nop, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "DCP", opcode_num: 0xC3, addr_mode: AddressingMode::IndirectX, addr_func: indirect_x, func: dcp, base_clocks: 8, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "CPY", opcode_num: 0xC4, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: cpy, base_clocks: 3, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "CMP", opcode_num: 0xC5, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: cmp, base_clocks: 3, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "DEC", opcode_num: 0xC6, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: dec, base_clocks: 5, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "DCP", opcode_num: 0xC7, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: dcp, base_clocks: 5, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "INY", opcode_num: 0xC8, addr_mode: AddressingMode::Implied, addr_func: implied, func: iny, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "CMP", opcode_num: 0xC9, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: cmp, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "DEX", opcode_num: 0xCA, addr_mode: AddressingMode::Implied, addr_func: implied, func: dex, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    illegal_op!(0xCB), // SBX
    Instruction{name: "CPY", opcode_num: 0xCC, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: cpy, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "CMP", opcode_num: 0xCD, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: cmp, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "DEC", opcode_num: 0xCE, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: dec, base_clocks: 6, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "DCP", opcode_num: 0xCF, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: dcp, base_clocks: 6, bytes: 3, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "BNE", opcode_num: 0xD0, addr_mode: AddressingMode::Relative, addr_func: relative, func: bne, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: true, is_illegal: false},
    Instruction{name: "CMP", opcode_num: 0xD1, addr_mode: AddressingMode::IndirectY, addr_func: indirect_y, func: cmp, base_clocks: 5, bytes: 2, has_extra_fetch_cycles: true, is_illegal: false},
    illegal_op!(0xD2), // JAM
    Instruction{name: "DCP", opcode_num: 0xD3, addr_mode: AddressingMode::IndirectY, addr_func: indirect_y, func: dcp, base_clocks: 8, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "NOP", opcode_num: 0xD4, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: nop, base_clocks: 4, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "CMP", opcode_num: 0xD5, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: cmp, base_clocks: 4, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "DEC", opcode_num: 0xD6, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: dec, base_clocks: 6, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "DCP", opcode_num: 0xD7, addr_mode: AddressingMode::Absolute, addr_func: zpage_x, func: dcp, base_clocks: 6, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "CLD", opcode_num: 0xD8, addr_mode: AddressingMode::Implied, addr_func: implied, func: cld, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "CMP", opcode_num: 0xD9, addr_mode: AddressingMode::AbsoluteY, addr_func: absolute_y, func: cmp, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: true, is_illegal: false},
    Instruction{name: "NOP", opcode_num: 0xDA, addr_mode: AddressingMode::Implied, addr_func: implied, func: nop, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "DCP", opcode_num: 0xDB, addr_mode: AddressingMode::AbsoluteY, addr_func: absolute_y, func: dcp, base_clocks: 7, bytes: 3, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "NOP", opcode_num: 0xDC, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: nop, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: true, is_illegal: true},
    Instruction{name: "CMP", opcode_num: 0xDD, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: cmp, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: true, is_illegal: false},
    Instruction{name: "DEC", opcode_num: 0xDE, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: dec, base_clocks: 7, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "DCP", opcode_num: 0xDF, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: dcp, base_clocks: 7, bytes: 3, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "CPX", opcode_num: 0xE0, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: cpx, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "SBC", opcode_num: 0xE1, addr_mode: AddressingMode::IndirectX, addr_func: indirect_x, func: sbc, base_clocks: 6, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "NOP", opcode_num: 0xE2, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: nop, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "ISC", opcode_num: 0xE3, addr_mode: AddressingMode::IndirectX, addr_func: indirect_x, func: isc, base_clocks: 8, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "CPX", opcode_num: 0xE4, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: cpx, base_clocks: 3, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "SBC", opcode_num: 0xE5, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: sbc, base_clocks: 3, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "INC", opcode_num: 0xE6, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: inc, base_clocks: 5, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ISC", opcode_num: 0xE7, addr_mode: AddressingMode::ZeroPage, addr_func: zero_page, func: isc, base_clocks: 5, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "INX", opcode_num: 0xE8, addr_mode: AddressingMode::Implied, addr_func: implied, func: inx, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "SBC", opcode_num: 0xE9, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: sbc, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "NOP", opcode_num: 0xEA, addr_mode: AddressingMode::Implied, addr_func: implied, func: nop, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "USBC", opcode_num: 0xEB, addr_mode: AddressingMode::Immediate, addr_func: immediate, func: sbc, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "CPX", opcode_num: 0xEC, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: cpx, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "SBC", opcode_num: 0xED, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: sbc, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "INC", opcode_num: 0xEE, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: inc, base_clocks: 6, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ISC", opcode_num: 0xEF, addr_mode: AddressingMode::Absolute, addr_func: absolute, func: isc, base_clocks: 6, bytes: 3, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "BEQ", opcode_num: 0xF0, addr_mode: AddressingMode::Relative, addr_func: relative, func: beq, base_clocks: 2, bytes: 2, has_extra_fetch_cycles: true, is_illegal: false},
    Instruction{name: "SBC", opcode_num: 0xF1, addr_mode: AddressingMode::IndirectY, addr_func: indirect_y, func: sbc, base_clocks: 5, bytes: 2, has_extra_fetch_cycles: true, is_illegal: false},
    illegal_op!(0xF2), // JAM
    Instruction{name: "ISC", opcode_num: 0xF3, addr_mode: AddressingMode::IndirectY, addr_func: indirect_y, func: isc, base_clocks: 8, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "NOP", opcode_num: 0xF4, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: nop, base_clocks: 4, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "SBC", opcode_num: 0xF5, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: sbc, base_clocks: 4, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "INC", opcode_num: 0xF6, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: inc, base_clocks: 6, bytes: 2, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ISC", opcode_num: 0xF7, addr_mode: AddressingMode::ZeroPageX, addr_func: zpage_x, func: isc, base_clocks: 6, bytes: 2, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "SED", opcode_num: 0xF8, addr_mode: AddressingMode::Implied, addr_func: implied, func: sed, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "SBC", opcode_num: 0xF9, addr_mode: AddressingMode::AbsoluteY, addr_func: absolute_y, func: sbc, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: true, is_illegal: false},
    Instruction{name: "NOP", opcode_num: 0xFA, addr_mode: AddressingMode::Implied, addr_func: implied, func: nop, base_clocks: 2, bytes: 1, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ISC", opcode_num: 0xFB, addr_mode: AddressingMode::AbsoluteY, addr_func: absolute_y, func: isc, base_clocks: 7, bytes: 3, has_extra_fetch_cycles: false, is_illegal: true},
    Instruction{name: "NOP", opcode_num: 0xFC, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: nop, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: true, is_illegal: true},
    Instruction{name: "SBC", opcode_num: 0xFD, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: sbc, base_clocks: 4, bytes: 3, has_extra_fetch_cycles: true, is_illegal: false},
    Instruction{name: "INC", opcode_num: 0xFE, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: inc, base_clocks: 7, bytes: 3, has_extra_fetch_cycles: false, is_illegal: false},
    Instruction{name: "ISC", opcode_num: 0xFF, addr_mode: AddressingMode::AbsoluteX, addr_func: absolute_x, func: isc, base_clocks: 7, bytes: 3, has_extra_fetch_cycles: false, is_illegal: true},
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
    pub addr_func: fn(cpu: &Cpu6502) -> (u16, usize),
    pub func: fn(cpu: &mut Cpu6502, address: u16) -> usize,
    pub base_clocks: usize,
    pub bytes: usize,
    pub has_extra_fetch_cycles: bool,
    pub is_illegal: bool,
}


// ADDRESSING MODES - Fetches data from the bus
// Returns:
//  - Address of the data needed for the instruction
//  - Number of extra CPU cycles taken to get data
//
// Note: Implied addressing mode has no return type because no data is needed for instructions
// with implied addressing mode.

// Accumulator - like implied, no extra data needed. Accumulator is used as data
fn accumulator(cpu: &Cpu6502) -> (u16, usize) {
    (0,0)
}
// Implied - no extra data needed for this instruction, read no extra bytes
fn implied(cpu: &Cpu6502) -> (u16, usize) {
    (0,0)
}
// Immediate - data immediatly follows instruction
fn immediate(cpu: &Cpu6502) -> (u16, usize) {
    (cpu.get_pc() + 1, 0)
}
// Absolute - The next 2 bytes are the address in memory of the data to retrieve
fn absolute(cpu: &Cpu6502) -> (u16, usize) {
    let abs_address = cpu.read_word(cpu.get_pc() + 1);
    (abs_address, 0)
}
// Indexed Addressing (X) - Like Absolute, but adds the x register to the abs address to get
// the "effective address," and uses that to fetch data from memory.
// Also known as Absolute X addressing.
fn absolute_x(cpu: &Cpu6502) -> (u16, usize) {
    let abs_address = cpu.read_word(cpu.get_pc() + 1);
    let effective_address = abs_address.wrapping_add(cpu.get_x_reg() as u16);

    let page_boundary_crossed: bool = (abs_address & 0xFF00) != (effective_address & 0xFF00);

    (effective_address, if page_boundary_crossed { 1 } else { 0 })
}
// Indexed Addressing (Y) - Like same as Indexed x, but used the y register instead.
// Also known as Absolute Y addressing.
fn absolute_y(cpu: &Cpu6502) -> (u16, usize) {
    let abs_address = cpu.read_word(cpu.get_pc() + 1);
    let effective_address = abs_address.wrapping_add(cpu.get_y_reg() as u16);

    let page_boundary_crossed: bool = (abs_address & 0xFF00) != (effective_address & 0xFF00);

    (effective_address, if page_boundary_crossed { 1 } else { 0 })
}
// Zero Page - Like absolute, but uses only 1 byte for address & uses 0x00 for the high byte of the address
fn zero_page(cpu: &Cpu6502) -> (u16, usize) {
    let zpage_address = cpu.read(cpu.get_pc() + 1) as u16;
    (zpage_address, 0)
}
// Indexed Addressing Zero-Page (X) - Like Indexed x, but uses only the single next byte as the
// low order byte of the absolute address and fills the top byte w/ 0x00. Then adds x to get
// the effective address. Note that the effective address will never go off the zero-page, if
// the address exceeds 0x00FF, it will loop back around to 0x0000.
// Also known as Zero Page X addressing.
fn zpage_x(cpu: &Cpu6502) -> (u16, usize) {
    let zpage_address = cpu.read(cpu.get_pc() + 1);
    let effective_zpage_address = zpage_address.wrapping_add(cpu.get_x_reg()) as u16;
    
    (effective_zpage_address, 0)
}
// Indexed Addressing Zero-Page (Y) - Like Indexed Z-Page x, but uses the y register instead
// Also known as Zero Page Y addressing.
fn zpage_y(cpu: &Cpu6502) -> (u16, usize) {
    let zpage_address = cpu.read(cpu.get_pc() + 1);
    let effective_zpage_address = zpage_address.wrapping_add(cpu.get_y_reg()) as u16;
    
    (effective_zpage_address, 0)
}
// Indirect Addressing - Uses the next 2 bytes as the abs address, then reads the byte that
// points to in memory and the one after (in LLHH order) and uses those as the effective
// address where the data will be read from.
// Note: This mode has a hardware bug where a page boundary cannot be crossed by 
// the reading of 2 bytes from abs_address, and therefore it can take no
// additional clock cycles.
fn indirect(cpu: &Cpu6502) -> (u16, usize) {
    let abs_address = cpu.read_word(cpu.get_pc() + 1);

    let effective_lo = cpu.read(abs_address) as u16;
    let effective_hi = if abs_address & 0xFF == 0xFF {
        cpu.read(abs_address & 0xFF00)
    } else {
        cpu.read(abs_address + 1)
    } as u16;

    let effective_address = (effective_hi << 8) | effective_lo;

    (effective_address, 0)
}
// Pre-Indexed Indirect Zero-Page (X) - Like Indexed Z-Page x, but as in Indirect addressing, another
// address is read from memory instead of the data. The address read is the effective
// address of the actual data. Note that if the z-page address is 0x00FF, then the bytes at
// 0x00FF and 0x0000 are read and used as low and high bytes of the effective address, respectively
fn indirect_x(cpu: &Cpu6502) -> (u16, usize) {
    let zpage_address = cpu.read(cpu.get_pc() + 1).wrapping_add(cpu.get_x_reg());
    let effective_address = cpu.read_zpage_word(zpage_address);
    
    (effective_address, 0)
}
// Post-Indexed Indirect Zero-Page (Y) - Like Indirect Indexed Z-Page x, but with two major differences:
// First, the y register is used instead of x. Second, the register is added to the address
// retrieved from the z-page, not the address used to access the z-page.
fn indirect_y(cpu: &Cpu6502) -> (u16, usize) {
    let zpage_address = cpu.read(cpu.get_pc() + 1);
    let abs_address = cpu.read_zpage_word(zpage_address);
    let effective_address = abs_address.wrapping_add(cpu.get_y_reg() as u16);
    
    let page_boundary_crossed = (abs_address & 0xFF00) != (effective_address & 0xFF00);

    (effective_address, if page_boundary_crossed { 1 } else { 0 })
}
// Relative Addressing - Data used is next byte
fn relative(cpu: &Cpu6502) -> (u16, usize) {
   (cpu.get_pc() + 1, 0)
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
fn adc(cpu: &mut Cpu6502, address: u16) -> usize {
    let data = cpu.read(address);

    let result = (data as u16) + (cpu.get_acc() as u16) + (cpu.status.carry() as u16);
    cpu.status.set_carry(result & 0xFF00 > 0);
    cpu.status.set_zero(result & 0xFF == 0);
    cpu.status.set_negative(result & 0x80 != 0);
    
    // Set V flag if acc and data are same sign, but result is different sign
    let a = cpu.get_acc() & 0x80 != 0;
    let r = (result & 0x80) != 0;
    let d = data & 0x80 != 0;
    cpu.status.set_overflow( !(a^d)&(a^r) ); // Trust, bro

    cpu.set_acc(result as u8);

    0
}
// AND - AND Memory with Accumulator
fn and(cpu: &mut Cpu6502, address: u16) -> usize {
    let data = cpu.read(address);

    let result = cpu.get_acc() & data;
    cpu.status.set_zero(result == 0);
    cpu.status.set_negative(result & 0x80 != 0);
    cpu.set_acc(result);
    0
}
// ASL - Shift Left One Bit (Accumulator version)
fn asl_acc(cpu: &mut Cpu6502, _address: u16) -> usize {
    let result = cpu.get_acc() << 1;
    cpu.status.set_carry((cpu.get_acc() & 0x80) != 0);
    cpu.status.set_zero(result == 0);
    cpu.status.set_negative(result & 0x80 != 0);
    cpu.set_acc(result);

    0
}
// ASL - Shift Left One Bit (Memory version)
fn asl_mem(cpu: &mut Cpu6502, address: u16) -> usize {
    let data = cpu.read(address);
    let result = data << 1;
    cpu.status.set_carry((data & 0x80) != 0);
    cpu.status.set_zero(result == 0);
    cpu.status.set_negative(result & 0x80 != 0);
    cpu.write(address, result);
    0
}
// BCC - Branch on Carry Clear
fn bcc(cpu: &mut Cpu6502, address: u16) -> usize {
    let offset = cpu.read(address) as i8;

    if !cpu.status.carry() {
        let prev_pc = cpu.get_pc();
        cpu.set_pc((cpu.get_pc() as i32 + offset as i32) as u16);
        
        let is_same_page = prev_pc & 0xFF00 == cpu.get_pc() & 0xFF00;

        if is_same_page {
            return 1;
        } else {
            return 2;
        }
    }
    0
}
// BCS - Branch on Carry Set
fn bcs(cpu: &mut Cpu6502, address: u16) -> usize {
    let offset = cpu.read(address) as i8;

    if cpu.status.carry() {
        let prev_pc = cpu.get_pc();
        cpu.set_pc((cpu.get_pc() as i32 + offset as i32) as u16);
        
        let is_same_page = prev_pc & 0xFF00 == cpu.get_pc() & 0xFF00;

        if is_same_page {
            return 1;
        } else {
            return 2;
        }
    }

    0
}
// BEQ - Branch on Equal (Zero flag set)
fn beq(cpu: &mut Cpu6502, address: u16) -> usize {
    let offset = cpu.read(address) as i8;

    if cpu.status.zero() {
        let prev_pc = cpu.get_pc();
        cpu.set_pc((cpu.get_pc() as i32 + offset as i32) as u16);
        
        let is_same_page = prev_pc & 0xFF00 == cpu.get_pc() & 0xFF00;

        if is_same_page {
            return 1;
        } else {
            return 2;
        }
    }
    
    0
}
// BIT - Test Bits in Memory with Accumulator
fn bit(cpu: &mut Cpu6502, address: u16) -> usize {
    let data = cpu.read(address);

    cpu.status.set_negative(data & 0x80 != 0);
    cpu.status.set_overflow(data & 0x40 != 0);
    cpu.status.set_zero(data & cpu.get_acc() == 0);
    0
}
// BMI - Branch on Result Minus (Negative flag set)
fn bmi(cpu: &mut Cpu6502, address: u16) -> usize {
    let offset = cpu.read(address) as i8;

    if cpu.status.negative() {
        let prev_pc = cpu.get_pc();
        cpu.set_pc((cpu.get_pc() as i32 + offset as i32) as u16);
        
        let is_same_page = prev_pc & 0xFF00 == cpu.get_pc() & 0xFF00;

        if is_same_page {
            return 1;
        } else {
            return 2;
        }
    }
    0
}
// BNE - Branch on Not Equal (Zero flag NOT set)
fn bne(cpu: &mut Cpu6502, address: u16) -> usize {
    let offset = cpu.read(address) as i8;

    if !cpu.status.zero() {
        let prev_pc = cpu.get_pc();
        cpu.set_pc((cpu.get_pc() as i32 + offset as i32) as u16);
        
        let is_same_page = prev_pc & 0xFF00 == cpu.get_pc() & 0xFF00;

        if is_same_page {
            return 1;
        } else {
            return 2;
        }
    }
    0
}
// BPL - Branch on Result Plus (Negative flag NOT set)
fn bpl(cpu: &mut Cpu6502, address: u16) -> usize {
    let offset = cpu.read(address) as i8;

    if !cpu.status.negative() {
        let prev_pc = cpu.get_pc();
        cpu.set_pc((cpu.get_pc() as i32 + offset as i32) as u16);
        
        let is_same_page = prev_pc & 0xFF00 == cpu.get_pc() & 0xFF00;

        if is_same_page {
            return 1;
        } else {
            return 2;
        }
    }
    0
}
// BRK - Force Break (Initiate interrupt)
fn brk(cpu: &mut Cpu6502, _address: u16) -> usize {
    cpu.irq();
    0
}
// BVC - Branch on Overflow clear
fn bvc(cpu: &mut Cpu6502, address: u16) -> usize {
    let offset = cpu.read(address) as i8;

    if !cpu.status.overflow() {
        let prev_pc = cpu.get_pc();
        cpu.set_pc((cpu.get_pc() as i32 + offset as i32) as u16);
        
        let is_same_page = prev_pc & 0xFF00 == cpu.get_pc() & 0xFF00;

        if is_same_page {
            return 1;
        } else {
            return 2;
        }
    }
    0
}
// BVS - Branch on Overflow set
fn bvs(cpu: &mut Cpu6502, address: u16) -> usize {
    let offset = cpu.read(address) as i8;

    if cpu.status.overflow() {
        let prev_pc = cpu.get_pc();
        cpu.set_pc((cpu.get_pc() as i32 + offset as i32) as u16);
        
        let is_same_page = prev_pc & 0xFF00 == cpu.get_pc() & 0xFF00;

        if is_same_page {
            return 1;
        } else {
            return 2;
        }
    }
    0
}
// CLC - Clear Carry Flag
fn clc(cpu: &mut Cpu6502, _address: u16) -> usize {
    cpu.status.set_carry(false);
    0
}
// CLD - Clear Decimal Mode
fn cld(cpu: &mut Cpu6502, _address: u16) -> usize {
    cpu.status.set_decimal(false);
    0
}
// CLI - Clear Interrupt Disable Bit
fn cli(cpu: &mut Cpu6502, _address: u16) -> usize {
    cpu.status.set_interrupt(false);
    0
}
// CLV - Clear Overflow Flag
fn clv(cpu: &mut Cpu6502, _address: u16) -> usize {
    cpu.status.set_overflow(false);
    0
}
// CMP - Compare Memory with Accumulator
fn cmp(cpu: &mut Cpu6502, address: u16) -> usize {
    let data = cpu.read(address);

    let result = (cpu.get_acc() as i16) - (data as i16);
    cpu.status.set_zero(result == 0);
    cpu.status.set_negative(result & 0x80 != 0);
    cpu.status.set_carry(result >= 0);
    0
}
// CPX - Compare Memory and Index X
fn cpx(cpu: &mut Cpu6502, address: u16) -> usize {
    let data = cpu.read(address);

    let result = (cpu.get_x_reg() as i16) - (data as i16);
    cpu.status.set_zero(result == 0);
    cpu.status.set_negative(result & 0x80 != 0);
    cpu.status.set_carry(result >= 0);
    0
}
// CPY - Compare Memory and Index Y
fn cpy(cpu: &mut Cpu6502, address: u16) -> usize {
    let data = cpu.read(address);

    let result = (cpu.get_y_reg() as i16) - (data as i16);
    cpu.status.set_zero(result == 0);
    cpu.status.set_negative(result & 0x80 != 0);
    cpu.status.set_carry(result >= 0);
    0
}
// DEC - Decrement Memory
fn dec(cpu: &mut Cpu6502, address: u16) -> usize {
    let data = cpu.read(address);

    let result = data.wrapping_sub(1);
    cpu.status.set_zero(result == 0);
    cpu.status.set_negative(result & 0x80 != 0);
    cpu.write(address, result);
    0
}
// DEX - Decrement X Register
fn dex(cpu: &mut Cpu6502, _address: u16) -> usize {
    let result = cpu.get_x_reg().wrapping_sub(1);
    
    cpu.status.set_zero(result == 0);
    cpu.status.set_negative(result & 0x80 != 0);
    cpu.set_x_reg(result);
    0
}
// DEY - Decrement Y Register
fn dey(cpu: &mut Cpu6502, _address: u16) -> usize {
    let result = cpu.get_y_reg().wrapping_sub(1);
    cpu.status.set_zero(result == 0);
    cpu.status.set_negative(result & 0x80 != 0);
    cpu.set_y_reg(result);
    0
}
// EOR - Exclusive OR
fn eor(cpu: &mut Cpu6502, address: u16) -> usize {
    let data = cpu.read(address);

    let result = cpu.get_acc() ^ data;
    cpu.status.set_zero(result == 0);
    cpu.status.set_negative(result & 0x80 != 0);
    cpu.set_acc(result);
    0
}
// INC - Increment Memory
fn inc(cpu: &mut Cpu6502, address: u16) -> usize {
    let data = cpu.read(address);

    // NOTE: The nesdev page on MMC1 (mapper 1) notes that the inc instr writes to
    // memory twice. Once before the increment (with the unchanged data), and once after.
    // This may be a source of error with mapper 1, as we aren't doing that rn.

    let result = data.wrapping_add(1);
    cpu.status.set_zero(result == 0);
    cpu.status.set_negative(result & 0x80 != 0);
    cpu.write(address, result);
    0
}
// INX - Increment X Register
fn inx(cpu: &mut Cpu6502, _address: u16) -> usize {
    let result = cpu.get_x_reg().wrapping_add(1);

    cpu.status.set_zero(result == 0);
    cpu.status.set_negative(result & 0x80 != 0);
    cpu.set_x_reg(result);
    0
}
// INY - Increment Y Register
fn iny(cpu: &mut Cpu6502, _address: u16) -> usize {
    let result = cpu.get_y_reg().wrapping_add(1);
    cpu.status.set_zero(result == 0);
    cpu.status.set_negative(result & 0x80 != 0);
    cpu.set_y_reg(result);
    0
}
// JMP - Jump
fn jmp(cpu: &mut Cpu6502, address: u16) -> usize {
    cpu.set_pc(address);
    0
}
// JSR - Jump to Subroutine
fn jsr(cpu: &mut Cpu6502, address: u16) -> usize {
    let return_point = cpu.get_pc().wrapping_sub(1); // Return point is pc - 1
    let hi = (return_point >> 8) as u8;
    let lo = return_point as u8;

    cpu.push_to_stack(hi);
    cpu.push_to_stack(lo);
    cpu.set_pc(address);

    0
}
// LDA - Load Accumulator
fn lda(cpu: &mut Cpu6502, address: u16) -> usize {
    let data = cpu.read(address);

    cpu.status.set_zero(data == 0);
    cpu.status.set_negative(data & 0x80 != 0);
    cpu.set_acc(data);
    0
}
// LDX - Load X Register
fn ldx(cpu: &mut Cpu6502, address: u16) -> usize {
    let data = cpu.read(address);

    cpu.status.set_zero(data == 0);
    cpu.status.set_negative(data & 0x80 != 0);
    cpu.set_x_reg(data);
    0
}
// LDY - Load Y Register
fn ldy(cpu: &mut Cpu6502, address: u16) -> usize {
    let data = cpu.read(address);

    cpu.status.set_zero(data == 0);
    cpu.status.set_negative(data & 0x80 != 0);
    cpu.set_y_reg(data);
    0
}
// LSR - Logical Shift Right (Accumulator version)
fn lsr_acc(cpu: &mut Cpu6502, _address: u16) -> usize {
    let result = cpu.get_acc() >> 1;
    cpu.status.set_carry(cpu.get_acc() & 0x01 == 1);
    cpu.status.set_zero(result == 0);
    cpu.status.set_negative(false); // result will always have bit 7 == 0
    cpu.set_acc(result);
    0
}
// LSR - Logical Shift Right (Memory version)
fn lsr_mem(cpu: &mut Cpu6502, address: u16) -> usize {
    let data = cpu.read(address);
    let result = data >> 1;
    cpu.status.set_carry(data & 0x01 == 1);
    cpu.status.set_zero(result == 0);
    cpu.status.set_negative(false); // result will always have bit 7 == 0
    cpu.write(address, result);
    0
}
// NOP - No Operation
fn nop(_: &mut Cpu6502, _address: u16) -> usize { 0 }
// ORA - Logical Inclusive OR
fn ora(cpu: &mut Cpu6502, address: u16) -> usize {
    let data = cpu.read(address);

    let result = cpu.get_acc() | data;
    cpu.status.set_zero(result == 0);
    cpu.status.set_negative(result & 0x80 != 0);
    cpu.set_acc(result);
    0
}
// PHA - Push Accumulator
fn pha(cpu: &mut Cpu6502, _address: u16) -> usize {
    cpu.push_to_stack(cpu.get_acc());
    0
}
// PHP - Push Processor Status
fn php(cpu: &mut Cpu6502, _address: u16) -> usize {
    // Bit 5 (unused flag) is always set to 1 when status pushed to stack
    // Bit 4 (break flag) is set when push to stk caused by php or brk
    cpu.push_to_stack(cpu.get_status() | 0x30);
    0
}
// PLA - Pull Accumulator
fn pla(cpu: &mut Cpu6502, _address: u16) -> usize {
    let result = cpu.pop_from_stack();
    cpu.status.set_zero(result == 0);
    cpu.status.set_negative(result & 0x80 != 0);
    cpu.set_acc(result);
    0
}
// PLP - Pull Processor Status
fn plp(cpu: &mut Cpu6502, _address: u16) -> usize {
    // Bit 5 is ignored when pulling into processor status
    // Bit 4 is cleared
    let data = cpu.pop_from_stack() & 0xCF;
    cpu.set_status(data | (cpu.get_status() & 0x20));
    0
}
// ROL - Rotate Left (Accumulator version)
fn rol_acc(cpu: &mut Cpu6502, _address: u16) -> usize {
    let result = (cpu.get_acc() << 1) | if cpu.status.carry() { 1 } else { 0 };
    cpu.status.set_carry(cpu.get_acc() >> 7 == 1); // old bit 7 becomes new carry
    cpu.status.set_zero(result == 0);
    cpu.status.set_negative(result & 0x80 != 0);
    cpu.set_acc(result);
    0
}
// ROL - Rotate Left (Memory version)
fn rol_mem(cpu: &mut Cpu6502, address: u16) -> usize {
    let data = cpu.read(address);
    let result = (data << 1) | if cpu.status.carry() { 1 } else { 0 };
    cpu.status.set_carry(data >> 7 == 1); // old bit 7 becomes new carry
    cpu.status.set_zero(result == 0);
    cpu.status.set_negative(result & 0x80 != 0);
    cpu.write(address, result);
    0
}
// ROR - Rotate Right (Accumulator version)
fn ror_acc(cpu: &mut Cpu6502, _address: u16) -> usize {
    let result = (if cpu.status.carry() { 1 } else { 0 } << 7) | (cpu.get_acc() >> 1);
    cpu.status.set_carry(cpu.get_acc() & 0x01 == 1); // old bit 0 becomes new carry
    cpu.status.set_zero(result == 0);
    cpu.status.set_negative(result & 0x80 != 0);
    cpu.set_acc(result);
    0
}
// ROR - Rotate Right (Memory version)
fn ror_mem(cpu: &mut Cpu6502, address: u16) -> usize {
    let data = cpu.read(address);
    let result = (if cpu.status.carry() { 1 } else { 0 } << 7) | (data >> 1);
    cpu.status.set_carry(data & 0x01 == 1); // old bit 0 becomes new carry
    cpu.status.set_zero(result == 0);
    cpu.status.set_negative(result & 0x80 != 0);
    cpu.write(address, result);
    0
}
// RTI - Return from Interrupt
fn rti(cpu: &mut Cpu6502, _address: u16) -> usize {
    // Restore processer status (Bit 5 ignored, bit 4 cleared)
    let prev_status = cpu.pop_from_stack() & 0xCF;
    cpu.set_status(prev_status | (cpu.get_status() & 0x20));
    // Return to previous PC
    let lo = cpu.pop_from_stack() as u16;
    let hi = cpu.pop_from_stack() as u16;
    cpu.set_pc((hi << 8) | lo);

    0
}
// RTS - Return from Subroutine
fn rts(cpu: &mut Cpu6502, _address: u16) -> usize {
    let lo = cpu.pop_from_stack() as u16;
    let hi = cpu.pop_from_stack() as u16;
    let new_pc = (hi << 8) | lo;
    cpu.set_pc(new_pc.wrapping_add(1));
    0
}
// SBC - Subtract with Carry
//  Note: instr 0xEB (illegal opcode) executes the same as 0xE9, which is legal.
//        0xEB is differentiated in the table only by the name "USBC" for "Undocumented SBC"
fn sbc(cpu: &mut Cpu6502, address: u16) -> usize {
    let data = cpu.read(address);
    // Add with carry: A + M + C
    // Sub with carry: A - M - (1 - C) == A + (-M - 1) + C
    let twos_comp = (data as i8).overflowing_mul(-1).0.wrapping_sub(1) as u8;
    // let twos_comp = (data as i8 * -1).wrapping_sub(1) as u8; // errors
    // let (twos_comp, _) = data.overflowing_neg();       // wrong behavior

    // ADC w/ two's compliment instead of original data
    let result = (twos_comp as u16) + (cpu.get_acc() as u16) + (cpu.status.carry() as u16);
    cpu.status.set_carry(result & 0xFF00 > 0);
    cpu.status.set_zero(result & 0xFF == 0);
    cpu.status.set_negative(result & 0x80 != 0);
    
    // Set V flag if acc and data are same sign, but result is different sign
    let a = cpu.get_acc() & 0x80 != 0;
    let r = (result & 0x80) != 0;
    let d = twos_comp & 0x80 != 0;
    cpu.status.set_overflow( !(a^d)&(a^r) ); // Trust, bro

    cpu.set_acc(result as u8);

    0
}
// SEC - Set Carry Flag
fn sec(cpu: &mut Cpu6502, _address: u16) -> usize {
    cpu.status.set_carry(true);
    0
}
// SED - Set Decimal Flag
fn sed(cpu: &mut Cpu6502, _address: u16) -> usize {
    cpu.status.set_decimal(true);
    0
}
// SEI - Set Interrupt Disable
fn sei(cpu: &mut Cpu6502, _address: u16) -> usize {
    cpu.status.set_interrupt(true);
    0
}
// STA - Store Accumulator
fn sta(cpu: &mut Cpu6502, address: u16) -> usize {
    cpu.write(address, cpu.get_acc());
    0
}
// STX - Store X Register
fn stx(cpu: &mut Cpu6502, address: u16) -> usize {
    cpu.write(address, cpu.get_x_reg());
    0
}
// STY - Store Y Register
fn sty(cpu: &mut Cpu6502, address: u16) -> usize {
    cpu.write(address, cpu.get_y_reg());
    0
}
// TAX - Transfer Accumulator to X
fn tax(cpu: &mut Cpu6502, _address: u16) -> usize {
    cpu.set_x_reg(cpu.get_acc());
    cpu.status.set_zero(cpu.get_x_reg() == 0);
    cpu.status.set_negative(cpu.get_x_reg() & 0x80 != 0);
    0
}
// TAY - Transfer Accumulator to Y
fn tay(cpu: &mut Cpu6502, _address: u16) -> usize {
    cpu.set_y_reg(cpu.get_acc());
    cpu.status.set_zero(cpu.get_y_reg() == 0);
    cpu.status.set_negative(cpu.get_y_reg() & 0x80 != 0);
    0
}
// TSX - Transfer Stack Pointer to X
fn tsx(cpu: &mut Cpu6502, _address: u16) -> usize {
    cpu.set_x_reg(cpu.get_sp());
    cpu.status.set_zero(cpu.get_x_reg() == 0);
    cpu.status.set_negative(cpu.get_x_reg() & 0x80 != 0);
    0
}
// TXA - Transfer X to Accumulator
fn txa(cpu: &mut Cpu6502, _address: u16) -> usize {
    cpu.set_acc(cpu.get_x_reg());
    cpu.status.set_zero(cpu.get_acc() == 0);
    cpu.status.set_negative(cpu.get_acc() & 0x80 != 0);
    0
}
// TXS - Transfer X to Stack Pointer
fn txs(cpu: &mut Cpu6502, _address: u16) -> usize {
    cpu.set_sp(cpu.get_x_reg());
    0
}
// TYA - Transfer Y to Accumulator
fn tya(cpu: &mut Cpu6502, _address: u16) -> usize {
    cpu.set_acc(cpu.get_y_reg());
    cpu.status.set_zero(cpu.get_acc() == 0);
    cpu.status.set_negative(cpu.get_acc() & 0x80 != 0);
    0
}


/// ======== ILLEGAL OPCODES ========

// INVALID OPCODE - An unimplemented opcode not recognized by the CPU.
//                  Placeholder for all unimplemented illegal opcodes.
fn xxx(_: &mut Cpu6502, _address: u16) -> usize { 0 }


// LAX - Load Accumulator and X Register
fn lax(cpu: &mut Cpu6502, address: u16) -> usize {
    let data = cpu.read(address);

    cpu.status.set_zero(data == 0);
    cpu.status.set_negative(data & 0x80 != 0);
    cpu.set_acc(data);
    cpu.set_x_reg(data);
    0
}

// SAX - Store Accumulator & X Register (bitwise acc & x)
fn sax(cpu: &mut Cpu6502, address: u16) -> usize {
    let result = cpu.get_acc() & cpu.get_x_reg();
    cpu.write(address, result);

    0
}

// Doing these illegal opcodes using other opcode functions may result in ppu
// registers being incorrectly incremented. definitely something to be aware of.

// DCP - Decrement Memory and Compare with Accumulator
fn dcp(cpu: &mut Cpu6502, address: u16) -> usize {
    dec(cpu, address);
    cmp(cpu, address);
    0
}

// ISC - Increment Memory and Subtract with Carry
fn isc(cpu: &mut Cpu6502, address: u16) -> usize {
    inc(cpu, address);    
    sbc(cpu, address);
    0
}

// SLO - Arithmetic Shift Left then Logical Inclusive OR
fn slo(cpu: &mut Cpu6502, address: u16) -> usize {
    asl_mem(cpu, address);
    ora(cpu, address);
    0
}

// RLA - Rotate Left then Logical AND with Accumulator
fn rla(cpu: &mut Cpu6502, address: u16) -> usize {
    rol_mem(cpu, address);
    and(cpu, address);
    0
}

// SRE - Logical Shift Right then "Exclusive OR" Memory with Accumulator
fn sre(cpu: &mut Cpu6502, address: u16) -> usize {
    lsr_mem(cpu, address);
    eor(cpu, address);
    0
}

// RRA - Rotate Right and Add Memory to Accumulator
fn rra(cpu: &mut Cpu6502, address: u16) -> usize {
    ror_mem(cpu, address);
    adc(cpu, address);
    0
}

// ANC - Bitwise AND Memory with Accumulator then Move Negative Flag to Carry Flag
fn anc(cpu: &mut Cpu6502, address: u16) -> usize {
    and(cpu, address);

    cpu.status.set_carry(cpu.status.negative());

    0
}

// ALR - Bitwise AND Memory with Accumulator then Logical Shift Right
fn asr(cpu: &mut Cpu6502, address: u16) -> usize {
    let data = cpu.read(address);

    let result = (data & cpu.get_acc()) >> 1 | (if cpu.status.carry() { 1 } else { 0 } << 7);
    cpu.status.set_carry(data & cpu.get_acc() & 0x01 == 1);
    cpu.status.set_zero(result == 0);
    cpu.status.set_negative(false);
    cpu.set_acc(result);

    0
}

// ARR - Bitwise AND Memory with Accumulator then Rotate Right
fn arr(cpu: &mut Cpu6502, address: u16) -> usize {
    let data = cpu.read(address);
    
    let result = (data & cpu.get_acc()) >> 1 | (if cpu.status.carry() { 1 } else { 0 } << 7);
    cpu.status.set_zero(result == 0);
    cpu.status.set_negative(false);
    cpu.set_acc(result);
    
    if cpu.status.decimal() {
        cpu.status.set_carry(result & 0x40 != 0);
        cpu.status.set_overflow(result & 0x40 != result & 0x20);
    } else {
        cpu.status.set_carry((result & 0xF0).wrapping_add(result & 0x10) > 0x50);
        cpu.status.set_overflow(result & 0x40 != data & 0x40);
    }

    0
}

// Not sure exactly how to go about ignoring offsets of addressing modes yet...
// // SHA Indirect Y - Store Accumulator Bitwise AND Index Register X Bitwise AND Memory
// fn sha_ind_y(cpu: &mut CPU, address: u16) -> usize {
//     let address = opcode_data.address.unwrap().wrapping_sub(cpu.get_y_reg() as u16); // "ignore" / undo the y offset of the address
//     let hi = (address >> 8) as u8;

//     let result = cpu.get_y_reg() & hi.wrapping_add(1);

//     cpu.write(address, result);

//     0
// }

// // SHY - Store Index Register Y Bitwise AND Value
// fn shy(cpu: &mut CPU, address: u16) -> usize {
//     let address = opcode_data.address.unwrap().wrapping_sub(cpu.get_x_reg() as u16); // "ignore" / undo the x offset of the address
//     let hi = (address >> 8) as u8;

//     let result = cpu.get_y_reg() & hi.wrapping_add(1);

//     cpu.write(address, result);

//     0
// }