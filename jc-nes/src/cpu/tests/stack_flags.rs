//! Stack operations and the flag setters/clearers.

use super::*;

#[test]
fn pha_pushes_to_the_stack_page() {
    let (mut cpu, cycles) = exec(&[0x48], |cpu| cpu.a = 0x42);
    assert_eq!(cpu.sp, SP_RESET - 1);
    assert_eq!(peek(&mut cpu, 0x01FD), 0x42, "stack lives at $0100 + SP");
    assert_eq!(cycles, 3);
}

#[test]
fn pla_pulls_and_sets_flags() {
    let (cpu, cycles) = exec(&[0x68], |cpu| {
        cpu.sp = 0xFC;
        poke(cpu, 0x01FD, &[0x80]);
    });
    assert_eq!(cpu.a, 0x80);
    assert_eq!(cpu.sp, 0xFD);
    assert_nz(&cpu, true, false);
    assert_eq!(cycles, 4);
}

#[test]
fn pla_sets_zero_on_a_zero_pull() {
    let (cpu, _) = exec(&[0x68], |cpu| {
        cpu.sp = 0xFC;
        poke(cpu, 0x01FD, &[0x00]);
    });
    assert_nz(&cpu, false, true);
}

#[test]
fn pha_and_pla_round_trip() {
    let mut cpu = cpu();
    poke(&mut cpu, PROG, &[0x48, 0x68]); // PHA, PLA
    cpu.a = 0x5A;
    step(&mut cpu);
    cpu.a = 0x00;
    step(&mut cpu);
    assert_eq!(cpu.a, 0x5A);
    assert_eq!(cpu.sp, SP_RESET, "stack must be balanced");
}

#[test]
fn php_pushes_with_b_and_bit_five_set() {
    // The B flag does not exist in the register; it is synthesised as 1 by PHP.
    let (mut cpu, cycles) = exec(&[0x08], |cpu| set_status(cpu, C));
    let pushed = peek(&mut cpu, 0x01FD);
    assert_eq!(pushed & B, B, "PHP sets bit 4 in the pushed byte");
    assert_eq!(pushed & U, U, "PHP sets bit 5 in the pushed byte");
    assert_eq!(pushed & C, C, "PHP preserves the real flags");
    assert_eq!(cycles, 3);
}

#[test]
fn plp_ignores_bit_four_and_forces_bit_five() {
    let (cpu, cycles) = exec(&[0x28], |cpu| {
        cpu.sp = 0xFC;
        poke(cpu, 0x01FD, &[0xFF]);
    });
    let restored = u8::from(cpu.status);
    assert_eq!(restored & B, 0, "PLP must not set the B flag");
    assert_eq!(restored & U, U, "bit 5 is always set");
    assert_eq!(cycles, 4);
}

#[test]
fn plp_restores_every_real_flag() {
    let (cpu, _) = exec(&[0x28], |cpu| {
        cpu.sp = 0xFC;
        set_status(cpu, 0);
        poke(cpu, 0x01FD, &[status_byte(N | V | D | I | Z | C)]);
    });
    assert!(cpu.status.negative);
    assert!(cpu.status.overflow);
    assert!(cpu.status.decimal);
    assert!(cpu.status.interrupt);
    assert!(cpu.status.zero);
    assert!(cpu.status.carry);
}

#[test]
fn php_and_plp_round_trip() {
    let mut cpu = cpu();
    poke(&mut cpu, PROG, &[0x08, 0x28]); // PHP, PLP
    set_status(&mut cpu, N | V | C);
    step(&mut cpu);
    set_status(&mut cpu, 0);
    step(&mut cpu);
    assert!(cpu.status.negative);
    assert!(cpu.status.overflow);
    assert!(cpu.status.carry);
    assert_eq!(cpu.sp, SP_RESET);
}

#[test]
fn the_stack_pointer_wraps_from_zero_on_push() {
    // Pushing with SP=$00 must wrap to $FF, writing at $0100. The stack is a
    // fixed 256-byte page; it never underflows out of it.
    let (cpu, _) = exec(&[0x48], |cpu| {
        cpu.a = 0x42;
        cpu.sp = 0x00;
    });
    assert_eq!(cpu.sp, 0xFF, "SP must wrap on push, not underflow");
}

#[test]
fn the_stack_pointer_wraps_from_ff_on_pull() {
    // Pulling with SP=$FF must wrap to $00, reading from $0100.
    let (cpu, _) = exec(&[0x68], |cpu| {
        cpu.sp = 0xFF;
        poke(cpu, 0x0100, &[0x42]);
    });
    assert_eq!(cpu.sp, 0x00, "SP must wrap on pull, not overflow");
    assert_eq!(cpu.a, 0x42);
}

#[test]
fn pushes_stay_inside_the_stack_page() {
    // With SP=$00 the byte lands at $0100, not at $0000 or anywhere outside
    // $0100-$01FF.
    let (mut cpu, _) = exec(&[0x48], |cpu| {
        cpu.a = 0x42;
        cpu.sp = 0x00;
    });
    assert_eq!(peek(&mut cpu, 0x0100), 0x42);
}

#[test]
fn flag_setters_and_clearers() {
    // Each is two cycles and touches exactly one flag.
    let (cpu, cycles) = exec(&[0x18], |cpu| set_status(cpu, C)); // CLC
    assert!(!cpu.status.carry);
    assert_eq!(cycles, 2);

    let (cpu, _) = exec(&[0x38], |cpu| set_status(cpu, 0)); // SEC
    assert!(cpu.status.carry);

    let (cpu, _) = exec(&[0x58], |cpu| set_status(cpu, I)); // CLI
    assert!(!cpu.status.interrupt);

    let (cpu, _) = exec(&[0x78], |cpu| set_status(cpu, 0)); // SEI
    assert!(cpu.status.interrupt);

    let (cpu, _) = exec(&[0xB8], |cpu| set_status(cpu, V)); // CLV
    assert!(!cpu.status.overflow);

    let (cpu, _) = exec(&[0xD8], |cpu| set_status(cpu, D)); // CLD
    assert!(!cpu.status.decimal);

    let (cpu, _) = exec(&[0xF8], |cpu| set_status(cpu, 0)); // SED
    assert!(cpu.status.decimal);
}

#[test]
fn clc_leaves_the_other_flags_untouched() {
    let (cpu, _) = exec(&[0x18], |cpu| set_status(cpu, N | V | Z | I | D | C));
    assert!(!cpu.status.carry);
    assert!(cpu.status.negative);
    assert!(cpu.status.overflow);
    assert!(cpu.status.zero);
    assert!(cpu.status.interrupt);
    assert!(cpu.status.decimal);
}

#[test]
fn there_is_no_instruction_that_sets_the_overflow_flag() {
    // CLV is the only V-flag instruction; SEV does not exist. $B8 clears it and
    // nothing in the official set sets it directly.
    let (cpu, _) = exec(&[0xB8], |cpu| set_status(cpu, V));
    assert!(!cpu.status.overflow);
}
