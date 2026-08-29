//! Addressing mode tests.
//!
//! Each mode is exercised through a load instruction so the effective address is
//! observable in a register, plus the cycle count so the page-cross penalty is
//! checked at the same time.

use super::*;

#[test]
fn immediate() {
    // LDA #$42
    let (cpu, cycles) = exec_bare(&[0xA9, 0x42]);
    assert_eq!(cpu.a, 0x42);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.pc, PROG + 2, "immediate is 2 bytes");
}

#[test]
fn zero_page() {
    // LDA $42
    let (cpu, cycles) = exec(&[0xA5, 0x42], |cpu| poke(cpu, 0x0042, &[0x37]));
    assert_eq!(cpu.a, 0x37);
    assert_eq!(cycles, 3);
    assert_eq!(cpu.pc, PROG + 2);
}

#[test]
fn zero_page_x() {
    // LDA $80,X with X=$0F -> $008F
    let (cpu, cycles) = exec(&[0xB5, 0x80], |cpu| {
        cpu.x = 0x0F;
        poke(cpu, 0x008F, &[0x37]);
    });
    assert_eq!(cpu.a, 0x37);
    assert_eq!(cycles, 4);
}

#[test]
fn zero_page_x_wraps_inside_page_zero() {
    // LDA $F0,X with X=$20 must resolve to $0010, never $0110.
    let (cpu, _) = exec(&[0xB5, 0xF0], |cpu| {
        cpu.x = 0x20;
        poke(cpu, 0x0010, &[0x11]);
        poke(cpu, 0x0110, &[0x22]);
    });
    assert_eq!(cpu.a, 0x11, "zp,X must wrap inside page zero");
}

#[test]
fn zero_page_y() {
    // LDX $80,Y with Y=$0F -> $008F
    let (cpu, cycles) = exec(&[0xB6, 0x80], |cpu| {
        cpu.y = 0x0F;
        poke(cpu, 0x008F, &[0x37]);
    });
    assert_eq!(cpu.x, 0x37);
    assert_eq!(cycles, 4);
}

#[test]
fn zero_page_y_wraps_inside_page_zero() {
    let (cpu, _) = exec(&[0xB6, 0xF0], |cpu| {
        cpu.y = 0x20;
        poke(cpu, 0x0010, &[0x11]);
        poke(cpu, 0x0110, &[0x22]);
    });
    assert_eq!(cpu.x, 0x11, "zp,Y must wrap inside page zero");
}

#[test]
fn absolute() {
    // LDA $1234
    let (cpu, cycles) = exec(&[0xAD, 0x34, 0x12], |cpu| poke(cpu, 0x1234, &[0x37]));
    assert_eq!(cpu.a, 0x37);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.pc, PROG + 3, "absolute is 3 bytes");
}

#[test]
fn absolute_x_without_page_cross() {
    // LDA $1200,X with X=$34 -> $1234, same page, no penalty.
    let (cpu, cycles) = exec(&[0xBD, 0x00, 0x12], |cpu| {
        cpu.x = 0x34;
        poke(cpu, 0x1234, &[0x37]);
    });
    assert_eq!(cpu.a, 0x37);
    assert_eq!(cycles, 4);
}

#[test]
fn absolute_x_with_page_cross_costs_one_more() {
    // LDA $12FF,X with X=$01 -> $1300, crosses into the next page.
    let (cpu, cycles) = exec(&[0xBD, 0xFF, 0x12], |cpu| {
        cpu.x = 0x01;
        poke(cpu, 0x1300, &[0x37]);
    });
    assert_eq!(cpu.a, 0x37);
    assert_eq!(cycles, 5, "abs,X read crossing a page costs 4+1");
}

#[test]
fn absolute_y_without_page_cross() {
    let (cpu, cycles) = exec(&[0xB9, 0x00, 0x12], |cpu| {
        cpu.y = 0x34;
        poke(cpu, 0x1234, &[0x37]);
    });
    assert_eq!(cpu.a, 0x37);
    assert_eq!(cycles, 4);
}

#[test]
fn absolute_y_with_page_cross_costs_one_more() {
    let (cpu, cycles) = exec(&[0xB9, 0xFF, 0x12], |cpu| {
        cpu.y = 0x01;
        poke(cpu, 0x1300, &[0x37]);
    });
    assert_eq!(cpu.a, 0x37);
    assert_eq!(cycles, 5, "abs,Y read crossing a page costs 4+1");
}

#[test]
fn absolute_x_wraps_around_the_address_space() {
    // LDA $FFFF,X with X=$02 -> $0001.
    let (cpu, _) = exec(&[0xBD, 0xFF, 0xFF], |cpu| {
        cpu.x = 0x02;
        poke(cpu, 0x0001, &[0x37]);
    });
    assert_eq!(cpu.a, 0x37, "abs,X must wrap at $FFFF");
}

#[test]
fn indexed_indirect() {
    // LDA ($20,X) with X=$04: pointer lives at $0024/$0025.
    let (cpu, cycles) = exec(&[0xA1, 0x20], |cpu| {
        cpu.x = 0x04;
        poke(cpu, 0x0024, &[0x34, 0x12]);
        poke(cpu, 0x1234, &[0x37]);
    });
    assert_eq!(cpu.a, 0x37);
    assert_eq!(cycles, 6);
}

#[test]
fn indexed_indirect_wraps_inside_page_zero() {
    // LDA ($20,X) with X=$FF: pointer bytes are at $001F and $0020, not $011F.
    let (cpu, _) = exec(&[0xA1, 0x20], |cpu| {
        cpu.x = 0xFF;
        poke(cpu, 0x001F, &[0x34]);
        poke(cpu, 0x0020, &[0x12]);
        poke(cpu, 0x1234, &[0x37]);
    });
    assert_eq!(cpu.a, 0x37, "(zp,X) pointer must wrap inside page zero");
}

#[test]
fn indirect_indexed() {
    // LDA ($20),Y with Y=$10: pointer at $0020 -> $1234, +$10 -> $1244.
    let (cpu, cycles) = exec(&[0xB1, 0x20], |cpu| {
        cpu.y = 0x10;
        poke(cpu, 0x0020, &[0x34, 0x12]);
        poke(cpu, 0x1244, &[0x37]);
    });
    assert_eq!(cpu.a, 0x37);
    assert_eq!(cycles, 5);
}

#[test]
fn indirect_indexed_with_page_cross_costs_one_more() {
    // pointer -> $12FF, +$01 -> $1300.
    let (cpu, cycles) = exec(&[0xB1, 0x20], |cpu| {
        cpu.y = 0x01;
        poke(cpu, 0x0020, &[0xFF, 0x12]);
        poke(cpu, 0x1300, &[0x37]);
    });
    assert_eq!(cpu.a, 0x37);
    assert_eq!(cycles, 6, "(zp),Y read crossing a page costs 5+1");
}

#[test]
fn indirect_indexed_pointer_wraps_inside_page_zero() {
    // LDA ($FF),Y: low byte at $00FF, high byte at $0000 (not $0100).
    let (cpu, _) = exec(&[0xB1, 0xFF], |cpu| {
        cpu.y = 0x00;
        poke(cpu, 0x00FF, &[0x34]);
        poke(cpu, 0x0000, &[0x12]);
        poke(cpu, 0x0100, &[0xFF]);
        poke(cpu, 0x1234, &[0x37]);
    });
    assert_eq!(cpu.a, 0x37, "(zp),Y pointer high byte must come from $0000");
}

#[test]
fn indirect_jump() {
    // JMP ($1234)
    let (cpu, cycles) = exec(&[0x6C, 0x34, 0x12], |cpu| poke(cpu, 0x1234, &[0x78, 0x56]));
    assert_eq!(cpu.pc, 0x5678);
    assert_eq!(cycles, 5);
}

#[test]
fn indirect_jump_page_boundary_hardware_bug() {
    // JMP ($12FF) reads the low byte from $12FF but wraps for the high byte,
    // taking it from $1200 rather than $1300. This bug is real hardware.
    let (cpu, _) = exec(&[0x6C, 0xFF, 0x12], |cpu| {
        poke(cpu, 0x12FF, &[0x78]);
        poke(cpu, 0x1200, &[0x56]);
        poke(cpu, 0x1300, &[0xAA]);
    });
    assert_eq!(
        cpu.pc, 0x5678,
        "JMP ($xxFF) must wrap to $xx00 for the high byte"
    );
}

#[test]
fn accumulator_mode_touches_no_memory() {
    // ASL A
    let (cpu, cycles) = exec(&[0x0A], |cpu| cpu.a = 0x21);
    assert_eq!(cpu.a, 0x42);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.pc, PROG + 1, "accumulator mode is 1 byte");
}

#[test]
fn implied_mode_is_one_byte() {
    // INX
    let (cpu, cycles) = exec(&[0xE8], |cpu| cpu.x = 0x41);
    assert_eq!(cpu.x, 0x42);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.pc, PROG + 1);
}

#[test]
fn program_counter_wraps_at_the_top_of_memory() {
    // An instruction fetched at $FFFF must wrap its operand fetches to $0000
    // rather than overflowing the program counter.
    let mut cpu = cpu();
    cpu.pc = 0xFFFF;
    poke(&mut cpu, 0xFFFF, &[0xA9]); // LDA #
    poke(&mut cpu, 0x0000, &[0x42]); // immediate operand, wrapped
    let cycles = step(&mut cpu);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.a, 0x42);
    assert_eq!(cpu.pc, 0x0001, "PC must wrap, not overflow");
}
