//! Bitwise logic, BIT, and the shift/rotate family.

use super::*;

#[test]
fn and_ora_eor_compute_and_set_flags() {
    // AND #$0F
    let (cpu, _) = exec(&[0x29, 0x0F], |cpu| cpu.a = 0xF0);
    assert_eq!(cpu.a, 0x00);
    assert_nz(&cpu, false, true);

    // ORA #$0F
    let (cpu, _) = exec(&[0x09, 0x0F], |cpu| cpu.a = 0xF0);
    assert_eq!(cpu.a, 0xFF);
    assert_nz(&cpu, true, false);

    // EOR #$FF
    let (cpu, _) = exec(&[0x49, 0xFF], |cpu| cpu.a = 0xF0);
    assert_eq!(cpu.a, 0x0F);
    assert_nz(&cpu, false, false);
}

#[test]
fn logic_leaves_carry_and_overflow_alone() {
    let (cpu, _) = exec(&[0x29, 0x0F], |cpu| {
        cpu.a = 0xF0;
        set_status(cpu, C | V);
    });
    assert!(cpu.status.carry, "AND must not touch C");
    assert!(cpu.status.overflow, "AND must not touch V");
}

#[test]
fn bit_takes_n_and_v_from_memory_and_z_from_the_mask() {
    // BIT $42 with M=$C0: N from bit 7, V from bit 6, Z from A & M.
    let (cpu, cycles) = exec(&[0x24, 0x42], |cpu| {
        cpu.a = 0x01;
        poke(cpu, 0x0042, &[0xC0]);
    });
    assert!(cpu.status.negative, "N comes from M bit 7");
    assert!(cpu.status.overflow, "V comes from M bit 6");
    assert!(cpu.status.zero, "A & M is zero");
    assert_eq!(cpu.a, 0x01, "BIT must not modify A");
    assert_eq!(cycles, 3);
}

#[test]
fn bit_clears_n_and_v_when_memory_has_them_clear() {
    let (cpu, _) = exec(&[0x24, 0x42], |cpu| {
        cpu.a = 0xFF;
        set_status(cpu, N | V);
        poke(cpu, 0x0042, &[0x3F]);
    });
    assert!(!cpu.status.negative);
    assert!(!cpu.status.overflow);
    assert!(!cpu.status.zero, "A & M is non-zero");
}

#[test]
fn bit_z_depends_on_a_not_just_memory() {
    // Same memory, different A: Z must follow the mask, not M alone.
    let (cpu, _) = exec(&[0x24, 0x42], |cpu| {
        cpu.a = 0x0F;
        poke(cpu, 0x0042, &[0xF0]);
    });
    assert!(cpu.status.zero, "$0F & $F0 == 0");
}

#[test]
fn asl_shifts_left_through_carry() {
    // ASL A on 0x81 -> 0x02 with C set from the old bit 7.
    let (cpu, _) = exec(&[0x0A], |cpu| cpu.a = 0x81);
    assert_eq!(cpu.a, 0x02);
    assert_nzc(&cpu, false, false, true);

    // ASL A on 0x40 -> 0x80, C clear, N set.
    let (cpu, _) = exec(&[0x0A], |cpu| cpu.a = 0x40);
    assert_eq!(cpu.a, 0x80);
    assert_nzc(&cpu, true, false, false);
}

#[test]
fn asl_does_not_shift_the_carry_in() {
    let (cpu, _) = exec(&[0x0A], |cpu| {
        cpu.a = 0x00;
        set_status(cpu, C);
    });
    assert_eq!(cpu.a, 0x00, "ASL shifts in a zero, never the carry");
    assert_nzc(&cpu, false, true, false);
}

#[test]
fn asl_memory_writes_back() {
    // ASL $42
    let (mut cpu, cycles) = exec(&[0x06, 0x42], |cpu| poke(cpu, 0x0042, &[0x81]));
    assert_eq!(peek(&mut cpu, 0x0042), 0x02);
    assert!(cpu.status.carry);
    assert_eq!(cycles, 5);
}

#[test]
fn lsr_shifts_right_and_always_clears_negative() {
    // LSR A on 0x01 -> 0x00 with C set.
    let (cpu, _) = exec(&[0x4A], |cpu| cpu.a = 0x01);
    assert_eq!(cpu.a, 0x00);
    assert_nzc(&cpu, false, true, true);

    // LSR A on 0xFF -> 0x7F; N must clear even though the input was negative.
    let (cpu, _) = exec(&[0x4A], |cpu| {
        cpu.a = 0xFF;
        set_status(cpu, N);
    });
    assert_eq!(cpu.a, 0x7F);
    assert!(!cpu.status.negative, "LSR always clears N");
    assert!(cpu.status.carry);
}

#[test]
fn rol_rotates_the_carry_in_at_bit_zero() {
    // ROL A on 0x80 with C clear -> 0x00, C set.
    let (cpu, _) = exec(&[0x2A], |cpu| {
        cpu.a = 0x80;
        set_status(cpu, 0);
    });
    assert_eq!(cpu.a, 0x00);
    assert_nzc(&cpu, false, true, true);

    // ROL A on 0x80 with C set -> 0x01, C set.
    let (cpu, _) = exec(&[0x2A], |cpu| {
        cpu.a = 0x80;
        set_status(cpu, C);
    });
    assert_eq!(cpu.a, 0x01);
    assert_nzc(&cpu, false, false, true);
}

#[test]
fn ror_rotates_the_carry_in_at_bit_seven() {
    // ROR A on 0x01 with C clear -> 0x00, C set.
    let (cpu, _) = exec(&[0x6A], |cpu| {
        cpu.a = 0x01;
        set_status(cpu, 0);
    });
    assert_eq!(cpu.a, 0x00);
    assert_nzc(&cpu, false, true, true);

    // ROR A on 0x00 with C set -> 0x80, C clear, N set.
    let (cpu, _) = exec(&[0x6A], |cpu| {
        cpu.a = 0x00;
        set_status(cpu, C);
    });
    assert_eq!(cpu.a, 0x80);
    assert_nzc(&cpu, true, false, false);
}

#[test]
fn rol_and_ror_memory_write_back() {
    // ROL $42 on 0x80 with C set -> 0x01
    let (mut cpu, cycles) = exec(&[0x26, 0x42], |cpu| {
        set_status(cpu, C);
        poke(cpu, 0x0042, &[0x80]);
    });
    assert_eq!(peek(&mut cpu, 0x0042), 0x01);
    assert!(cpu.status.carry);
    assert_eq!(cycles, 5);

    // ROR $42 on 0x01 with C set -> 0x80
    let (mut cpu, cycles) = exec(&[0x66, 0x42], |cpu| {
        set_status(cpu, C);
        poke(cpu, 0x0042, &[0x01]);
    });
    assert_eq!(peek(&mut cpu, 0x0042), 0x80);
    assert!(cpu.status.carry);
    assert_eq!(cycles, 5);
}

#[test]
fn read_modify_write_absolute_x_is_always_seven_cycles() {
    // ASL $12FF,X with X=1 crosses a page but RMW never takes a penalty: the
    // extra cycle is already unconditional.
    let (mut cpu, cycles) = exec(&[0x1E, 0xFF, 0x12], |cpu| {
        cpu.x = 0x01;
        poke(cpu, 0x1300, &[0x40]);
    });
    assert_eq!(peek(&mut cpu, 0x1300), 0x80);
    assert_eq!(cycles, 7, "ASL abs,X is always 7 cycles");
}
