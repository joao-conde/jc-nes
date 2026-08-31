//! Loads, stores and register transfers.

use super::*;

#[test]
fn lda_sets_negative_and_zero() {
    let (cpu, _) = exec_bare(&[0xA9, 0x00]);
    assert_eq!(cpu.a, 0x00);
    assert_nz(&cpu, false, true);

    let (cpu, _) = exec_bare(&[0xA9, 0x80]);
    assert_eq!(cpu.a, 0x80);
    assert_nz(&cpu, true, false);

    let (cpu, _) = exec_bare(&[0xA9, 0x7F]);
    assert_eq!(cpu.a, 0x7F);
    assert_nz(&cpu, false, false);
}

#[test]
fn ldx_sets_negative_and_zero() {
    let (cpu, _) = exec_bare(&[0xA2, 0x00]);
    assert_nz(&cpu, false, true);

    let (cpu, _) = exec_bare(&[0xA2, 0x80]);
    assert_eq!(cpu.x, 0x80);
    assert_nz(&cpu, true, false);
}

#[test]
fn ldy_sets_negative_and_zero() {
    let (cpu, _) = exec_bare(&[0xA0, 0x00]);
    assert_nz(&cpu, false, true);

    let (cpu, _) = exec_bare(&[0xA0, 0x80]);
    assert_eq!(cpu.y, 0x80);
    assert_nz(&cpu, true, false);
}

#[test]
fn loads_do_not_disturb_carry_or_overflow() {
    let (cpu, _) = exec(&[0xA9, 0x00], |cpu| set_status(cpu, C | V));
    assert!(cpu.status.carry, "LDA must not touch C");
    assert!(cpu.status.overflow, "LDA must not touch V");
}

#[test]
fn sta_writes_accumulator_and_sets_no_flags() {
    // STA $1234
    let (mut cpu, cycles) = exec(&[0x8D, 0x34, 0x12], |cpu| {
        cpu.a = 0x00;
        set_status(cpu, 0);
    });
    assert_eq!(peek(&mut cpu, 0x1234), 0x00);
    assert_eq!(cycles, 4);
    assert!(
        !cpu.status.zero,
        "STA must not set Z even when storing zero"
    );
    assert!(!cpu.status.negative, "STA must not set N");
}

#[test]
fn stx_and_sty_write_their_registers() {
    // STX $42
    let (mut cpu, _) = exec(&[0x86, 0x42], |cpu| cpu.x = 0xAB);
    assert_eq!(peek(&mut cpu, 0x0042), 0xAB);

    // STY $42
    let (mut cpu, _) = exec(&[0x84, 0x42], |cpu| cpu.y = 0xCD);
    assert_eq!(peek(&mut cpu, 0x0042), 0xCD);
}

#[test]
fn sta_absolute_x_never_takes_a_page_cross_penalty() {
    // Stores always pay the fixed 5 cycles: the extra read cycle is unconditional
    // on a write, so there is no penalty to add.
    let (mut cpu, cycles) = exec(&[0x9D, 0xFF, 0x12], |cpu| {
        cpu.a = 0x37;
        cpu.x = 0x01;
    });
    assert_eq!(peek(&mut cpu, 0x1300), 0x37);
    assert_eq!(cycles, 5, "STA abs,X is always 5 cycles");
}

#[test]
fn sta_indirect_indexed_never_takes_a_page_cross_penalty() {
    let (mut cpu, cycles) = exec(&[0x91, 0x20], |cpu| {
        cpu.a = 0x37;
        cpu.y = 0x01;
        poke(cpu, 0x0020, &[0xFF, 0x12]);
    });
    assert_eq!(peek(&mut cpu, 0x1300), 0x37);
    assert_eq!(cycles, 6, "STA (zp),Y is always 6 cycles");
}

#[test]
fn tax_tay_txa_tya_set_negative_and_zero() {
    // TAX
    let (cpu, _) = exec(&[0xAA], |cpu| cpu.a = 0x80);
    assert_eq!(cpu.x, 0x80);
    assert_nz(&cpu, true, false);

    // TAY
    let (cpu, _) = exec(&[0xA8], |cpu| cpu.a = 0x00);
    assert_eq!(cpu.y, 0x00);
    assert_nz(&cpu, false, true);

    // TXA
    let (cpu, _) = exec(&[0x8A], |cpu| cpu.x = 0x7F);
    assert_eq!(cpu.a, 0x7F);
    assert_nz(&cpu, false, false);

    // TYA
    let (cpu, _) = exec(&[0x98], |cpu| cpu.y = 0xFF);
    assert_eq!(cpu.a, 0xFF);
    assert_nz(&cpu, true, false);
}

#[test]
fn tsx_transfers_stack_pointer_and_sets_flags() {
    let (cpu, _) = exec(&[0xBA], |cpu| cpu.sp = 0x00);
    assert_eq!(cpu.x, 0x00);
    assert_nz(&cpu, false, true);

    let (cpu, _) = exec(&[0xBA], |cpu| cpu.sp = 0x80);
    assert_eq!(cpu.x, 0x80);
    assert_nz(&cpu, true, false);
}

#[test]
fn txs_transfers_x_but_sets_no_flags() {
    // TXS is the one transfer that leaves the flags alone.
    let (cpu, _) = exec(&[0x9A], |cpu| {
        cpu.x = 0x00;
        set_status(cpu, 0);
    });
    assert_eq!(cpu.sp, 0x00);
    assert!(!cpu.status.zero, "TXS must not set Z");
    assert!(!cpu.status.negative, "TXS must not set N");
}
