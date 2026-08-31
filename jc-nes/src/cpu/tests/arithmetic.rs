//! ADC / SBC / compares / increments and decrements.

use super::*;

/// The canonical signed-overflow matrix for ADC, with carry in clear.
///
/// `(a, m, result, carry_out, overflow)`
const ADC_OVERFLOW: [(u8, u8, u8, bool, bool); 8] = [
    (0x50, 0x10, 0x60, false, false), //   80 +   16 =   96
    (0x50, 0x50, 0xA0, false, true),  //   80 +   80 =  160, overflows to -96
    (0x50, 0x90, 0xE0, false, false), //   80 + -112 =  -32
    (0x50, 0xD0, 0x20, true, false),  //   80 +  -48 =   32
    (0xD0, 0x10, 0xE0, false, false), //  -48 +   16 =  -32
    (0xD0, 0x50, 0x20, true, false),  //  -48 +   80 =   32
    (0xD0, 0x90, 0x60, true, true),   //  -48 + -112 = -160, overflows to +96
    (0xD0, 0xD0, 0xA0, true, false),  //  -48 +  -48 =  -96
];

/// The same matrix for SBC, with carry in set (i.e. no borrow).
///
/// `(a, m, result, carry_out, overflow)`
const SBC_OVERFLOW: [(u8, u8, u8, bool, bool); 8] = [
    (0x50, 0xF0, 0x60, false, false), //   80 -  -16 =   96
    (0x50, 0xB0, 0xA0, false, true),  //   80 -  -80 =  160, overflows to -96
    (0x50, 0x70, 0xE0, false, false), //   80 -  112 =  -32
    (0x50, 0x30, 0x20, true, false),  //   80 -   48 =   32
    (0xD0, 0xF0, 0xE0, false, false), //  -48 -  -16 =  -32
    (0xD0, 0xB0, 0x20, true, false),  //  -48 -  -80 =   32
    (0xD0, 0x70, 0x60, true, true),   //  -48 -  112 = -160, overflows to +96
    (0xD0, 0x30, 0xA0, true, false),  //  -48 -   48 =  -96
];

#[test]
fn adc_signed_overflow_matrix() {
    for (a, m, result, carry_out, overflow) in ADC_OVERFLOW {
        let (cpu, _) = exec(&[0x69, m], |cpu| {
            cpu.a = a;
            set_status(cpu, 0);
        });
        assert_eq!(cpu.a, result, "ADC #{m:#04X} with A={a:#04X}");
        assert_eq!(cpu.status.carry, carry_out, "C after ADC {a:#04X}+{m:#04X}");
        assert_eq!(
            cpu.status.overflow, overflow,
            "V after ADC {a:#04X}+{m:#04X}"
        );
    }
}

#[test]
fn adc_adds_the_incoming_carry() {
    let (cpu, _) = exec(&[0x69, 0x01], |cpu| {
        cpu.a = 0x01;
        set_status(cpu, C);
    });
    assert_eq!(cpu.a, 0x03, "ADC must add the carry in");
}

#[test]
fn adc_sets_zero_only_on_a_zero_result() {
    let (cpu, _) = exec(&[0x69, 0x01], |cpu| {
        cpu.a = 0xFF;
        set_status(cpu, 0);
    });
    assert_eq!(cpu.a, 0x00);
    assert_nzc(&cpu, false, true, true);
}

#[test]
fn sbc_signed_overflow_matrix() {
    for (a, m, result, carry_out, overflow) in SBC_OVERFLOW {
        let (cpu, _) = exec(&[0xE9, m], |cpu| {
            cpu.a = a;
            set_status(cpu, C);
        });
        assert_eq!(cpu.a, result, "SBC #{m:#04X} with A={a:#04X}");
        assert_eq!(cpu.status.carry, carry_out, "C after SBC {a:#04X}-{m:#04X}");
        assert_eq!(
            cpu.status.overflow, overflow,
            "V after SBC {a:#04X}-{m:#04X}"
        );
    }
}

#[test]
fn sbc_borrows_when_carry_is_clear() {
    // With C clear the subtraction borrows, so 0x05 - 0x01 - 1 = 0x03.
    let (cpu, _) = exec(&[0xE9, 0x01], |cpu| {
        cpu.a = 0x05;
        set_status(cpu, 0);
    });
    assert_eq!(cpu.a, 0x03, "SBC must subtract the borrow");
    assert!(cpu.status.carry, "no borrow out of 5-1-1");
}

#[test]
fn sbc_clears_carry_when_the_result_borrows() {
    let (cpu, _) = exec(&[0xE9, 0x01], |cpu| {
        cpu.a = 0x00;
        set_status(cpu, C);
    });
    assert_eq!(cpu.a, 0xFF);
    assert!(!cpu.status.carry, "0 - 1 must borrow, clearing C");
}

#[test]
fn decimal_flag_is_ignored_on_the_2a03() {
    // The NES CPU has its BCD circuitry disabled: SED must not change how ADC
    // computes.
    let (cpu, _) = exec(&[0x69, 0x01], |cpu| {
        cpu.a = 0x09;
        set_status(cpu, D);
    });
    assert_eq!(cpu.a, 0x0A, "ADC stays binary with D set on the 2A03");
}

#[test]
fn cmp_sets_carry_zero_and_negative() {
    // A > M
    let (cpu, _) = exec(&[0xC9, 0x10], |cpu| cpu.a = 0x20);
    assert_nzc(&cpu, false, false, true);

    // A == M
    let (cpu, _) = exec(&[0xC9, 0x20], |cpu| cpu.a = 0x20);
    assert_nzc(&cpu, false, true, true);

    // A < M, and the difference has bit 7 set
    let (cpu, _) = exec(&[0xC9, 0x40], |cpu| cpu.a = 0x20);
    assert_nzc(&cpu, true, false, false);
}

#[test]
fn cmp_does_not_modify_the_accumulator() {
    let (cpu, _) = exec(&[0xC9, 0x10], |cpu| cpu.a = 0x20);
    assert_eq!(cpu.a, 0x20, "CMP is non-destructive");
}

#[test]
fn cmp_compares_unsigned() {
    // 0x00 vs 0x80: unsigned, A is smaller, so C clears.
    let (cpu, _) = exec(&[0xC9, 0x80], |cpu| cpu.a = 0x00);
    assert!(!cpu.status.carry, "CMP is an unsigned comparison");

    // 0x80 vs 0x00: A is larger unsigned, so C sets.
    let (cpu, _) = exec(&[0xC9, 0x00], |cpu| cpu.a = 0x80);
    assert!(cpu.status.carry);
}

#[test]
fn cpx_and_cpy_compare_their_registers() {
    // CPX #$10 with X=$20
    let (cpu, _) = exec(&[0xE0, 0x10], |cpu| cpu.x = 0x20);
    assert_nzc(&cpu, false, false, true);

    // CPY #$20 with Y=$20
    let (cpu, _) = exec(&[0xC0, 0x20], |cpu| cpu.y = 0x20);
    assert_nzc(&cpu, false, true, true);
}

#[test]
fn inc_and_dec_wrap_and_set_flags() {
    // INC $42 on 0xFF wraps to 0x00.
    let (mut cpu, cycles) = exec(&[0xE6, 0x42], |cpu| poke(cpu, 0x0042, &[0xFF]));
    assert_eq!(peek(&mut cpu, 0x0042), 0x00);
    assert_nz(&cpu, false, true);
    assert_eq!(cycles, 5);

    // DEC $42 on 0x00 wraps to 0xFF.
    let (mut cpu, cycles) = exec(&[0xC6, 0x42], |cpu| poke(cpu, 0x0042, &[0x00]));
    assert_eq!(peek(&mut cpu, 0x0042), 0xFF);
    assert_nz(&cpu, true, false);
    assert_eq!(cycles, 5);
}

#[test]
fn inc_and_dec_leave_carry_alone() {
    let (cpu, _) = exec(&[0xE6, 0x42], |cpu| {
        set_status(cpu, C);
        poke(cpu, 0x0042, &[0xFF]);
    });
    assert!(cpu.status.carry, "INC must not touch C");
}

#[test]
fn inx_dex_iny_dey_wrap() {
    let (cpu, _) = exec(&[0xE8], |cpu| cpu.x = 0xFF);
    assert_eq!(cpu.x, 0x00);
    assert_nz(&cpu, false, true);

    let (cpu, _) = exec(&[0xCA], |cpu| cpu.x = 0x00);
    assert_eq!(cpu.x, 0xFF);
    assert_nz(&cpu, true, false);

    let (cpu, _) = exec(&[0xC8], |cpu| cpu.y = 0xFF);
    assert_eq!(cpu.y, 0x00);

    let (cpu, _) = exec(&[0x88], |cpu| cpu.y = 0x00);
    assert_eq!(cpu.y, 0xFF);
}
