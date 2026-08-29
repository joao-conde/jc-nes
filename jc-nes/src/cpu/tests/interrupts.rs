//! Reset and NMI sequences.

use super::*;

#[test]
fn reset_loads_the_program_counter_from_fffc() {
    let mut cpu = cpu();
    poke(&mut cpu, 0xFFFC, &[0x34, 0x12]);
    cpu.reset();
    assert_eq!(cpu.pc, 0x1234, "reset vector is $FFFC/$FFFD, little endian");
}

#[test]
fn reset_settles_the_stack_pointer_at_fd() {
    let mut cpu = cpu();
    cpu.sp = 0x00;
    cpu.reset();
    assert_eq!(cpu.sp, 0xFD);
}

#[test]
fn reset_disables_interrupts() {
    // The reset sequence sets the interrupt disable flag, so a handler runs with
    // IRQs masked until it chooses otherwise. Without this a game's init code
    // can be interrupted before its first SEI.
    let mut cpu = cpu();
    cpu.reset();
    assert!(cpu.status.interrupt, "reset must set the I flag");
}

#[test]
fn reset_leaves_bit_five_set() {
    let mut cpu = cpu();
    cpu.reset();
    assert_eq!(
        u8::from(cpu.status) & U,
        U,
        "bit 5 of the status register is always set"
    );
}

#[test]
fn nmi_vectors_through_fffa() {
    let mut cpu = cpu();
    poke(&mut cpu, 0xFFFA, &[0x34, 0x12]);
    cpu.pc = 0x8000;
    cpu.nmi();
    assert_eq!(cpu.pc, 0x1234, "NMI vector is $FFFA/$FFFB");
}

#[test]
fn nmi_pushes_the_return_address_and_status() {
    let mut cpu = cpu();
    cpu.pc = 0x8123;
    set_status(&mut cpu, C | Z);
    cpu.nmi();

    assert_eq!(cpu.sp, SP_RESET - 3, "NMI pushes PCH, PCL and P");
    assert_eq!(
        peek(&mut cpu, 0x01FD),
        0x81,
        "high byte of the return address"
    );
    assert_eq!(
        peek(&mut cpu, 0x01FC),
        0x23,
        "low byte of the return address"
    );

    let pushed = peek(&mut cpu, 0x01FB);
    assert_eq!(pushed & B, 0, "an interrupt pushes P with B clear");
    assert_eq!(pushed & U, U, "bit 5 is always set when pushed");
    assert_eq!(pushed & C, C, "the real flags are preserved");
    assert_eq!(pushed & Z, Z);
}

#[test]
fn nmi_disables_further_interrupts() {
    let mut cpu = cpu();
    set_status(&mut cpu, 0);
    cpu.nmi();
    assert!(cpu.status.interrupt, "NMI sets the I flag");
}

#[test]
fn nmi_takes_seven_cycles() {
    let mut cpu = cpu();
    cpu.cycle = 0;
    cpu.nmi();
    assert_eq!(cpu.cycle, 7, "the NMI sequence is 7 cycles on hardware");
}

#[test]
fn nmi_and_rti_round_trip() {
    let mut cpu = cpu();
    poke(&mut cpu, 0xFFFA, &[0x00, 0x90]); // NMI handler at $9000
    poke(&mut cpu, 0x9000, &[0x40]); // RTI
    cpu.pc = 0x8123;
    set_status(&mut cpu, C);

    cpu.nmi();
    assert_eq!(cpu.pc, 0x9000);

    step(&mut cpu);
    assert_eq!(cpu.pc, 0x8123, "RTI must return to the interrupted address");
    assert_eq!(cpu.sp, SP_RESET, "the stack must be balanced");
    assert!(cpu.status.carry, "flags must be restored");
}
