//! Machine-checked opcode tables: decode coverage, base cycle counts, and the
//! page-cross penalty.
//!
//! These are the tests that catch systematic mistakes — a whole addressing mode
//! wired up with the wrong cost, or a block of opcodes that never got decoded —
//! which per-instruction tests tend to miss.

use super::*;

/// Every official opcode except the eight branches, whose cost depends on
/// whether they are taken; those are covered in `branch_jump.rs`.
///
/// `(opcode, mnemonic, base cycles)`
#[rustfmt::skip]
const OFFICIAL: [(u8, &str, u8); 143] = [
    (0x00, "BRK",         7), (0x01, "ORA (zp,X)",   6), (0x05, "ORA zp",      3),
    (0x06, "ASL zp",      5), (0x08, "PHP",          3), (0x09, "ORA #",       2),
    (0x0A, "ASL A",       2), (0x0D, "ORA abs",      4), (0x0E, "ASL abs",     6),
    (0x11, "ORA (zp),Y",  5), (0x15, "ORA zp,X",     4), (0x16, "ASL zp,X",    6),
    (0x18, "CLC",         2), (0x19, "ORA abs,Y",    4), (0x1D, "ORA abs,X",   4),
    (0x1E, "ASL abs,X",   7), (0x20, "JSR",          6), (0x21, "AND (zp,X)",  6),
    (0x24, "BIT zp",      3), (0x25, "AND zp",       3), (0x26, "ROL zp",      5),
    (0x28, "PLP",         4), (0x29, "AND #",        2), (0x2A, "ROL A",       2),
    (0x2C, "BIT abs",     4), (0x2D, "AND abs",      4), (0x2E, "ROL abs",     6),
    (0x31, "AND (zp),Y",  5), (0x35, "AND zp,X",     4), (0x36, "ROL zp,X",    6),
    (0x38, "SEC",         2), (0x39, "AND abs,Y",    4), (0x3D, "AND abs,X",   4),
    (0x3E, "ROL abs,X",   7), (0x40, "RTI",          6), (0x41, "EOR (zp,X)",  6),
    (0x45, "EOR zp",      3), (0x46, "LSR zp",       5), (0x48, "PHA",         3),
    (0x49, "EOR #",       2), (0x4A, "LSR A",        2), (0x4C, "JMP abs",     3),
    (0x4D, "EOR abs",     4), (0x4E, "LSR abs",      6), (0x51, "EOR (zp),Y",  5),
    (0x55, "EOR zp,X",    4), (0x56, "LSR zp,X",     6), (0x58, "CLI",         2),
    (0x59, "EOR abs,Y",   4), (0x5D, "EOR abs,X",    4), (0x5E, "LSR abs,X",   7),
    (0x60, "RTS",         6), (0x61, "ADC (zp,X)",   6), (0x65, "ADC zp",      3),
    (0x66, "ROR zp",      5), (0x68, "PLA",          4), (0x69, "ADC #",       2),
    (0x6A, "ROR A",       2), (0x6C, "JMP (ind)",    5), (0x6D, "ADC abs",     4),
    (0x6E, "ROR abs",     6), (0x71, "ADC (zp),Y",   5), (0x75, "ADC zp,X",    4),
    (0x76, "ROR zp,X",    6), (0x78, "SEI",          2), (0x79, "ADC abs,Y",   4),
    (0x7D, "ADC abs,X",   4), (0x7E, "ROR abs,X",    7), (0x81, "STA (zp,X)",  6),
    (0x84, "STY zp",      3), (0x85, "STA zp",       3), (0x86, "STX zp",      3),
    (0x88, "DEY",         2), (0x8A, "TXA",          2), (0x8C, "STY abs",     4),
    (0x8D, "STA abs",     4), (0x8E, "STX abs",      4), (0x91, "STA (zp),Y",  6),
    (0x94, "STY zp,X",    4), (0x95, "STA zp,X",     4), (0x96, "STX zp,Y",    4),
    (0x98, "TYA",         2), (0x99, "STA abs,Y",    5), (0x9A, "TXS",         2),
    (0x9D, "STA abs,X",   5), (0xA0, "LDY #",        2), (0xA1, "LDA (zp,X)",  6),
    (0xA2, "LDX #",       2), (0xA4, "LDY zp",       3), (0xA5, "LDA zp",      3),
    (0xA6, "LDX zp",      3), (0xA8, "TAY",          2), (0xA9, "LDA #",       2),
    (0xAA, "TAX",         2), (0xAC, "LDY abs",      4), (0xAD, "LDA abs",     4),
    (0xAE, "LDX abs",     4), (0xB1, "LDA (zp),Y",   5), (0xB4, "LDY zp,X",    4),
    (0xB5, "LDA zp,X",    4), (0xB6, "LDX zp,Y",     4), (0xB8, "CLV",         2),
    (0xB9, "LDA abs,Y",   4), (0xBA, "TSX",          2), (0xBC, "LDY abs,X",   4),
    (0xBD, "LDA abs,X",   4), (0xBE, "LDX abs,Y",    4), (0xC0, "CPY #",       2),
    (0xC1, "CMP (zp,X)",  6), (0xC4, "CPY zp",       3), (0xC5, "CMP zp",      3),
    (0xC6, "DEC zp",      5), (0xC8, "INY",          2), (0xC9, "CMP #",       2),
    (0xCA, "DEX",         2), (0xCC, "CPY abs",      4), (0xCD, "CMP abs",     4),
    (0xCE, "DEC abs",     6), (0xD1, "CMP (zp),Y",   5), (0xD5, "CMP zp,X",    4),
    (0xD6, "DEC zp,X",    6), (0xD8, "CLD",          2), (0xD9, "CMP abs,Y",   4),
    (0xDD, "CMP abs,X",   4), (0xDE, "DEC abs,X",    7), (0xE0, "CPX #",       2),
    (0xE1, "SBC (zp,X)",  6), (0xE4, "CPX zp",       3), (0xE5, "SBC zp",      3),
    (0xE6, "INC zp",      5), (0xE8, "INX",          2), (0xE9, "SBC #",       2),
    (0xEA, "NOP",         2), (0xEC, "CPX abs",      4), (0xED, "SBC abs",     4),
    (0xEE, "INC abs",     6), (0xF1, "SBC (zp),Y",   5), (0xF5, "SBC zp,X",    4),
    (0xF6, "INC zp,X",    6), (0xF8, "SED",          2), (0xF9, "SBC abs,Y",   4),
    (0xFD, "SBC abs,X",   4), (0xFE, "INC abs,X",    7),
];

/// Undocumented opcodes jc-nes decodes, with their hardware cycle counts.
#[rustfmt::skip]
const ILLEGAL: [(u8, &str, u8); 66] = [
    (0x03, "SLO (zp,X)",  8), (0x07, "SLO zp",      5), (0x0F, "SLO abs",     6),
    (0x13, "SLO (zp),Y",  8), (0x17, "SLO zp,X",    6), (0x1B, "SLO abs,Y",   7),
    (0x1F, "SLO abs,X",   7), (0x23, "RLA (zp,X)",  8), (0x27, "RLA zp",      5),
    (0x2F, "RLA abs",     6), (0x33, "RLA (zp),Y",  8), (0x37, "RLA zp,X",    6),
    (0x3B, "RLA abs,Y",   7), (0x3F, "RLA abs,X",   7), (0x43, "SRE (zp,X)",  8),
    (0x47, "SRE zp",      5), (0x4F, "SRE abs",     6), (0x53, "SRE (zp),Y",  8),
    (0x57, "SRE zp,X",    6), (0x5B, "SRE abs,Y",   7), (0x5F, "SRE abs,X",   7),
    (0x63, "RRA (zp,X)",  8), (0x67, "RRA zp",      5), (0x6F, "RRA abs",     6),
    (0x73, "RRA (zp),Y",  8), (0x77, "RRA zp,X",    6), (0x7B, "RRA abs,Y",   7),
    (0x7F, "RRA abs,X",   7), (0x83, "SAX (zp,X)",  6), (0x87, "SAX zp",      3),
    (0x8F, "SAX abs",     4), (0x97, "SAX zp,Y",    4), (0xA3, "LAX (zp,X)",  6),
    (0xA7, "LAX zp",      3), (0xAB, "LXA #",       2), (0xAF, "LAX abs",     4),
    (0xB3, "LAX (zp),Y",  5), (0xB7, "LAX zp,Y",    4), (0xBF, "LAX abs,Y",   4),
    (0xC3, "DCP (zp,X)",  8), (0xC7, "DCP zp",      5), (0xCF, "DCP abs",     6),
    (0xD3, "DCP (zp),Y",  8), (0xD7, "DCP zp,X",    6), (0xDB, "DCP abs,Y",   7),
    (0xDF, "DCP abs,X",   7), (0xE3, "ISC (zp,X)",  8), (0xE7, "ISC zp",      5),
    (0xEB, "USBC #",      2), (0xEF, "ISC abs",     6), (0xF3, "ISC (zp),Y",  8),
    (0xF7, "ISC zp,X",    6), (0xFB, "ISC abs,Y",   7), (0xFF, "ISC abs,X",   7),
    (0x0B, "ANC #",       2), (0x2B, "ANC #",       2), (0x4B, "ALR #",       2),
    (0x6B, "ARR #",       2), (0x8B, "ANE #",       2), (0xCB, "SBX #",       2),
    (0xBB, "LAS abs,Y",   4), (0x93, "SHA (zp),Y",  6), (0x9B, "TAS abs,Y",   5),
    (0x9C, "SHY abs,X",   5), (0x9E, "SHX abs,Y",   5), (0x9F, "SHA abs,Y",   5),
];

/// Read instructions that gain one cycle when indexing crosses a page.
///
/// `(opcode, mnemonic, mode)` where mode is `X` for abs,X, `Y` for abs,Y and
/// `I` for (zp),Y.
#[rustfmt::skip]
const PAGE_CROSS: [(u8, &str, char); 28] = [
    (0x1D, "ORA abs,X",  'X'), (0x19, "ORA abs,Y",  'Y'), (0x11, "ORA (zp),Y", 'I'),
    (0x3D, "AND abs,X",  'X'), (0x39, "AND abs,Y",  'Y'), (0x31, "AND (zp),Y", 'I'),
    (0x5D, "EOR abs,X",  'X'), (0x59, "EOR abs,Y",  'Y'), (0x51, "EOR (zp),Y", 'I'),
    (0x7D, "ADC abs,X",  'X'), (0x79, "ADC abs,Y",  'Y'), (0x71, "ADC (zp),Y", 'I'),
    (0xBD, "LDA abs,X",  'X'), (0xB9, "LDA abs,Y",  'Y'), (0xB1, "LDA (zp),Y", 'I'),
    (0xDD, "CMP abs,X",  'X'), (0xD9, "CMP abs,Y",  'Y'), (0xD1, "CMP (zp),Y", 'I'),
    (0xFD, "SBC abs,X",  'X'), (0xF9, "SBC abs,Y",  'Y'), (0xF1, "SBC (zp),Y", 'I'),
    (0xBC, "LDY abs,X",  'X'), (0xBE, "LDX abs,Y",  'Y'),
    (0xBF, "LAX abs,Y",  'Y'), (0xB3, "LAX (zp),Y", 'I'),
    (0x1C, "NOP abs,X",  'X'), (0x3C, "NOP abs,X",  'X'),
    (0xBB, "LAS abs,Y",  'Y'),
];

/// The JAM/KIL opcodes, which halt a real 6502 permanently.
///
/// These are the only opcodes that may stop the CPU. Every other value decodes
/// to a working instruction, which `process_opcode` enforces at compile time by
/// matching all 256 with no wildcard arm.
#[rustfmt::skip]
const JAM: [u8; 12] = [
    0x02, 0x12, 0x22, 0x32, 0x42, 0x52,
    0x62, 0x72, 0x92, 0xB2, 0xD2, 0xF2,
];

/// Stack pointer used for the table scans.
///
/// Parked mid-page so that the pulls in RTS/RTI/PLA/PLP and the pushes in
/// JSR/BRK/PHA/PHP stay clear of both ends: this file is measuring cycle counts,
/// and stack wrapping is covered on its own in `stack_flags.rs`.
const SP_MIDPAGE: u8 = 0x80;

/// Run one opcode with operands chosen so no addressing mode crosses a page.
fn cycles_for(opcode: u8) -> u8 {
    exec(&[opcode, 0x10, 0x20], |cpu| {
        cpu.x = 0;
        cpu.y = 0;
        cpu.sp = SP_MIDPAGE;
    })
    .1
}

#[test]
fn official_opcodes_have_the_right_base_cycle_count() {
    let mut wrong = Vec::new();
    for (opcode, name, expected) in OFFICIAL {
        let actual = cycles_for(opcode);
        if actual != expected {
            wrong.push(format!(
                "{opcode:#04X} {name}: expected {expected}, got {actual}"
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "{} official opcode(s) have the wrong cycle count:\n  {}",
        wrong.len(),
        wrong.join("\n  ")
    );
}

#[test]
fn illegal_opcodes_have_the_right_base_cycle_count() {
    let mut wrong = Vec::new();
    for (opcode, name, expected) in ILLEGAL {
        let actual = cycles_for(opcode);
        if actual != expected {
            wrong.push(format!(
                "{opcode:#04X} {name}: expected {expected}, got {actual}"
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "{} illegal opcode(s) have the wrong cycle count:\n  {}",
        wrong.len(),
        wrong.join("\n  ")
    );
}

#[test]
fn indexed_reads_cost_one_more_cycle_when_they_cross_a_page() {
    let mut wrong = Vec::new();

    for (opcode, name, mode) in PAGE_CROSS {
        // Same page: base + 0.  Crossing: base + 1.
        let same = match mode {
            'X' => exec(&[opcode, 0x00, 0x12], |cpu| cpu.x = 0x01).1,
            'Y' => exec(&[opcode, 0x00, 0x12], |cpu| cpu.y = 0x01).1,
            _ => {
                exec(&[opcode, 0x20], |cpu| {
                    cpu.y = 0x01;
                    poke(cpu, 0x0020, &[0x00, 0x12]);
                })
                .1
            }
        };
        let crossed = match mode {
            'X' => exec(&[opcode, 0xFF, 0x12], |cpu| cpu.x = 0x01).1,
            'Y' => exec(&[opcode, 0xFF, 0x12], |cpu| cpu.y = 0x01).1,
            _ => {
                exec(&[opcode, 0x20], |cpu| {
                    cpu.y = 0x01;
                    poke(cpu, 0x0020, &[0xFF, 0x12]);
                })
                .1
            }
        };

        if crossed != same + 1 {
            wrong.push(format!(
                "{opcode:#04X} {name}: {same} cycles same-page, {crossed} crossing (expected {})",
                same + 1
            ));
        }
    }

    assert!(
        wrong.is_empty(),
        "{} opcode(s) miss the page-cross penalty:\n  {}",
        wrong.len(),
        wrong.join("\n  ")
    );
}

#[test]
fn stores_and_read_modify_writes_never_take_a_page_cross_penalty() {
    // These already pay for the extra read unconditionally, so indexing across a
    // page must not add anything.
    let fixed = [
        (0x9Du8, "STA abs,X", 5u8, 'X'),
        (0x99, "STA abs,Y", 5, 'Y'),
        (0x1E, "ASL abs,X", 7, 'X'),
        (0x3E, "ROL abs,X", 7, 'X'),
        (0x5E, "LSR abs,X", 7, 'X'),
        (0x7E, "ROR abs,X", 7, 'X'),
        (0xDE, "DEC abs,X", 7, 'X'),
        (0xFE, "INC abs,X", 7, 'X'),
    ];

    for (opcode, name, expected, mode) in fixed {
        let crossed = if mode == 'X' {
            exec(&[opcode, 0xFF, 0x12], |cpu| cpu.x = 0x01).1
        } else {
            exec(&[opcode, 0xFF, 0x12], |cpu| cpu.y = 0x01).1
        };
        assert_eq!(crossed, expected, "{opcode:#04X} {name} crossing a page");
    }
}

#[test]
fn a_jam_opcode_halts_the_cpu() {
    // JAM stops the processor fetching. It claims no cycles and cannot advance
    // PC, so the only faithful response is to stop.
    let mut cpu = cpu();
    poke(&mut cpu, PROG, &[0x02]);

    cpu.clock();
    assert_eq!(cpu.pc, PROG, "PC stays on the offending byte");

    // Clocking a jammed CPU must be a safe no-op: before this was modelled, the
    // cycle counter went past zero here, panicking in debug and wrapping to 255
    // in release so the CPU span on the same byte forever.
    for _ in 0..1000 {
        cpu.clock();
    }
    assert_eq!(cpu.pc, PROG, "a jammed CPU makes no further progress");
}

#[test]
fn reset_clears_a_jam() {
    let mut cpu = cpu();
    poke(&mut cpu, PROG, &[0x02]);
    cpu.clock();

    poke(&mut cpu, 0xFFFC, &[0x00, 0x90]);
    cpu.reset();
    poke(&mut cpu, 0x9000, &[0xE8]); // INX

    // Reset itself costs cycles, so drain those before the first fetch lands.
    for _ in 0..16 {
        cpu.clock();
    }

    assert_eq!(cpu.x, 1, "the CPU runs again after a reset");
}

#[test]
fn only_jam_opcodes_halt_the_cpu() {
    // Every other opcode must leave the CPU running.
    for opcode in 0x00u8..=0xFF {
        if JAM.contains(&opcode) {
            continue;
        }
        let mut cpu = cpu();
        cpu.sp = SP_MIDPAGE;
        poke(&mut cpu, PROG, &[opcode, 0x10, 0x20]);
        cpu.clock();
        assert!(!cpu.jammed, "opcode {:#04X} jammed the CPU", opcode);
    }
}
