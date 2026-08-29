//! Undocumented opcodes that real cartridges rely on.
//!
//! The combined read-modify-write forms all follow the same shape: perform the
//! memory operation, write it back, then fold the new value into A with the
//! matching ALU operation.

use super::*;

#[test]
fn lax_loads_both_a_and_x() {
    // LAX $42
    let (cpu, cycles) = exec(&[0xA7, 0x42], |cpu| poke(cpu, 0x0042, &[0x80]));
    assert_eq!(cpu.a, 0x80);
    assert_eq!(cpu.x, 0x80, "LAX loads X as well as A");
    assert_nz(&cpu, true, false);
    assert_eq!(cycles, 3);
    assert_eq!(cpu.pc, PROG + 2, "LAX zp is 2 bytes");
}

#[test]
fn lax_sets_zero() {
    let (cpu, _) = exec(&[0xA7, 0x42], |cpu| poke(cpu, 0x0042, &[0x00]));
    assert_eq!(cpu.a, 0x00);
    assert_eq!(cpu.x, 0x00);
    assert_nz(&cpu, false, true);
}

#[test]
fn sax_stores_a_and_x_without_touching_flags() {
    // SAX $42
    let (mut cpu, cycles) = exec(&[0x87, 0x42], |cpu| {
        cpu.a = 0xF0;
        cpu.x = 0x3C;
        set_status(cpu, 0);
    });
    assert_eq!(peek(&mut cpu, 0x0042), 0x30, "SAX stores A & X");
    assert_eq!(cpu.a, 0xF0, "A is unchanged");
    assert_eq!(cpu.x, 0x3C, "X is unchanged");
    assert!(!cpu.status.zero, "SAX sets no flags");
    assert!(!cpu.status.negative);
    assert_eq!(cycles, 3);
}

#[test]
fn slo_shifts_left_then_ors_into_a() {
    // SLO $42: M=$81 -> $02 with C set, then A |= $02.
    let (mut cpu, cycles) = exec(&[0x07, 0x42], |cpu| {
        cpu.a = 0x10;
        set_status(cpu, 0);
        poke(cpu, 0x0042, &[0x81]);
    });
    assert_eq!(
        peek(&mut cpu, 0x0042),
        0x02,
        "memory holds the shifted value"
    );
    assert_eq!(cpu.a, 0x12, "A |= the shifted value");
    assert!(cpu.status.carry, "C comes from the shifted-out bit 7");
    assert_eq!(cycles, 5);
    assert_eq!(cpu.pc, PROG + 2);
}

#[test]
fn rla_rotates_left_then_ands_into_a() {
    // RLA $42: M=$80 with C set -> $01 with C set, then A &= $01.
    let (mut cpu, cycles) = exec(&[0x27, 0x42], |cpu| {
        cpu.a = 0x03;
        set_status(cpu, C);
        poke(cpu, 0x0042, &[0x80]);
    });
    assert_eq!(peek(&mut cpu, 0x0042), 0x01);
    assert_eq!(cpu.a, 0x01, "A &= the rotated value");
    assert!(cpu.status.carry);
    assert_eq!(cycles, 5);
}

#[test]
fn sre_shifts_right_then_eors_into_a() {
    // SRE $42: M=$03 -> $01 with C set, then A ^= $01.
    let (mut cpu, cycles) = exec(&[0x47, 0x42], |cpu| {
        cpu.a = 0x10;
        set_status(cpu, 0);
        poke(cpu, 0x0042, &[0x03]);
    });
    assert_eq!(peek(&mut cpu, 0x0042), 0x01);
    assert_eq!(cpu.a, 0x11, "A ^= the shifted value");
    assert!(cpu.status.carry, "C comes from the shifted-out bit 0");
    assert_eq!(cycles, 5);
}

#[test]
fn rra_rotates_right_then_adds_into_a() {
    // RRA $42: M=$02 with C clear -> $01 with C clear, then A = A + $01 + 0.
    let (mut cpu, cycles) = exec(&[0x67, 0x42], |cpu| {
        cpu.a = 0x10;
        set_status(cpu, 0);
        poke(cpu, 0x0042, &[0x02]);
    });
    assert_eq!(peek(&mut cpu, 0x0042), 0x01);
    assert_eq!(cpu.a, 0x11, "A = A + the rotated value + carry");
    assert_eq!(cycles, 5);
}

#[test]
fn rra_feeds_the_rotate_carry_into_the_addition() {
    // M=$03 with C clear rotates to $01 and sets C, so the ADC adds that carry:
    // A = $10 + $01 + 1 = $12.
    let (mut cpu, _) = exec(&[0x67, 0x42], |cpu| {
        cpu.a = 0x10;
        set_status(cpu, 0);
        poke(cpu, 0x0042, &[0x03]);
    });
    assert_eq!(peek(&mut cpu, 0x0042), 0x01);
    assert_eq!(cpu.a, 0x12, "the rotate's carry out feeds the add");
}

#[test]
fn dcp_decrements_then_compares() {
    // DCP $42: M=$43 -> $42, then CMP against A=$42 gives equality.
    let (mut cpu, cycles) = exec(&[0xC7, 0x42], |cpu| {
        cpu.a = 0x42;
        set_status(cpu, 0);
        poke(cpu, 0x0042, &[0x43]);
    });
    assert_eq!(peek(&mut cpu, 0x0042), 0x42);
    assert_eq!(cpu.a, 0x42, "DCP does not modify A");
    assert_nzc(&cpu, false, true, true);
    assert_eq!(cycles, 5);
}

#[test]
fn dcp_compares_against_the_decremented_value() {
    // M=$01 -> $00, A=$00: equal, so Z and C set.
    let (cpu, _) = exec(&[0xC7, 0x42], |cpu| {
        cpu.a = 0x00;
        set_status(cpu, 0);
        poke(cpu, 0x0042, &[0x01]);
    });
    assert_nzc(&cpu, false, true, true);
}

#[test]
fn isc_increments_then_subtracts() {
    // ISC $42: M=$41 -> $42, then SBC with C set: A = $50 - $42 = $0E.
    let (mut cpu, cycles) = exec(&[0xE7, 0x42], |cpu| {
        cpu.a = 0x50;
        set_status(cpu, C);
        poke(cpu, 0x0042, &[0x41]);
    });
    assert_eq!(peek(&mut cpu, 0x0042), 0x42);
    assert_eq!(cpu.a, 0x0E, "A -= the incremented value");
    assert!(cpu.status.carry, "no borrow");
    assert_eq!(cycles, 5);
}

#[test]
fn usbc_is_an_alias_for_sbc_immediate() {
    // $EB behaves exactly like $E9.
    let (usbc, usbc_cycles) = exec(&[0xEB, 0x01], |cpu| {
        cpu.a = 0x05;
        set_status(cpu, C);
    });
    let (sbc, sbc_cycles) = exec(&[0xE9, 0x01], |cpu| {
        cpu.a = 0x05;
        set_status(cpu, C);
    });
    assert_eq!(usbc.a, sbc.a);
    assert_eq!(usbc.a, 0x04);
    assert_eq!(usbc_cycles, sbc_cycles);
    assert_eq!(usbc.pc, sbc.pc, "$EB is 2 bytes like $E9");
}

#[test]
fn unofficial_nops_consume_their_operands_without_side_effects() {
    // (opcode, total bytes, cycles)
    let cases = [
        (0x1Au8, 1u16, 2u8), // NOP implied
        (0x80, 2, 2),        // NOP immediate
        (0x04, 2, 3),        // NOP zero page
        (0x14, 2, 4),        // NOP zero page,X
        (0x0C, 3, 4),        // NOP absolute
        (0x1C, 3, 4),        // NOP absolute,X
    ];

    for (opcode, bytes, cycles) in cases {
        let (cpu, actual) = exec(&[opcode, 0x10, 0x20], |cpu| {
            cpu.a = 0x11;
            cpu.x = 0x00;
            cpu.y = 0x33;
            set_status(cpu, C | V);
        });
        assert_eq!(cpu.pc, PROG + bytes, "opcode {opcode:#04X} byte count");
        assert_eq!(actual, cycles, "opcode {opcode:#04X} cycle count");
        assert_eq!(cpu.a, 0x11, "opcode {opcode:#04X} must not touch A");
        assert_eq!(cpu.y, 0x33, "opcode {opcode:#04X} must not touch Y");
        assert!(cpu.status.carry, "opcode {:#04X} must not touch C", opcode);
        assert!(
            cpu.status.overflow,
            "opcode {:#04X} must not touch V",
            opcode
        );
    }
}

#[test]
fn all_unofficial_nop_encodings_are_recognised() {
    // Every documented NOP encoding must decode; landing on the catch-all arm
    // returns zero cycles and would hang the CPU on a real ROM.
    let encodings = [
        0x1Au8, 0x3A, 0x5A, 0x7A, 0xDA, 0xFA, // implied
        0x80, 0x82, 0x89, 0xC2, 0xE2, // immediate
        0x04, 0x44, 0x64, // zero page
        0x14, 0x34, 0x54, 0x74, 0xD4, 0xF4, // zero page,X
        0x0C, // absolute
        0x1C, 0x3C, 0x5C, 0x7C, 0xDC, 0xFC, // absolute,X
    ];

    let missing: Vec<String> = encodings
        .iter()
        .filter(|&&op| exec(&[op, 0x10, 0x20], |_| {}).1 == 0)
        .map(|op| format!("{op:#04X}"))
        .collect();

    assert!(
        missing.is_empty(),
        "unimplemented NOP encodings: {}",
        missing.join(", ")
    );
}
