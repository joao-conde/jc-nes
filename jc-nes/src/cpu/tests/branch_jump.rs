//! Branches, jumps, subroutine calls and returns.

use super::*;

/// `(opcode, mnemonic, flag mask, taken when the flag is set)`
const BRANCHES: [(u8, &str, u8, bool); 8] = [
    (0x10, "BPL", N, false),
    (0x30, "BMI", N, true),
    (0x50, "BVC", V, false),
    (0x70, "BVS", V, true),
    (0x90, "BCC", C, false),
    (0xB0, "BCS", C, true),
    (0xD0, "BNE", Z, false),
    (0xF0, "BEQ", Z, true),
];

/// Assemble `prog` at `addr`, run `setup`, execute one instruction.
fn exec_at(addr: u16, prog: &[u8], setup: impl FnOnce(&mut super::Cpu)) -> (super::Cpu, u8) {
    let mut cpu = cpu();
    poke(&mut cpu, addr, prog);
    cpu.pc = addr;
    setup(&mut cpu);
    let cycles = step(&mut cpu);
    (cpu, cycles)
}

#[test]
fn every_branch_respects_its_condition() {
    for (opcode, name, mask, taken_when_set) in BRANCHES {
        // Flag set.
        let (cpu, cycles) = exec(&[opcode, 0x02], |cpu| set_status(cpu, mask));
        if taken_when_set {
            assert_eq!(cpu.pc, PROG + 4, "{name} should branch when flag is set");
            assert_eq!(cycles, 3, "{name} taken costs 3");
        } else {
            assert_eq!(cpu.pc, PROG + 2, "{name} should fall through when set");
            assert_eq!(cycles, 2, "{name} not taken costs 2");
        }

        // Flag clear.
        let (cpu, cycles) = exec(&[opcode, 0x02], |cpu| set_status(cpu, 0));
        if taken_when_set {
            assert_eq!(cpu.pc, PROG + 2, "{name} should fall through when clear");
            assert_eq!(cycles, 2, "{name} not taken costs 2");
        } else {
            assert_eq!(cpu.pc, PROG + 4, "{name} should branch when flag is clear");
            assert_eq!(cycles, 3, "{name} taken costs 3");
        }
    }
}

#[test]
fn a_taken_branch_crossing_a_page_costs_one_more() {
    // BEQ at $80FD: the operand sits at $80FE, the next instruction at $80FF,
    // and the target is $8101 - a different page, so 2 + 1 + 1 = 4 cycles.
    let (cpu, cycles) = exec_at(0x80FD, &[0xF0, 0x02], |cpu| set_status(cpu, Z));
    assert_eq!(cpu.pc, 0x8101);
    assert_eq!(cycles, 4, "taken branch crossing a page costs 4");
}

#[test]
fn a_taken_branch_within_a_page_costs_three() {
    let (cpu, cycles) = exec_at(0x8010, &[0xF0, 0x02], |cpu| set_status(cpu, Z));
    assert_eq!(cpu.pc, 0x8014);
    assert_eq!(cycles, 3);
}

#[test]
fn branches_go_backwards_with_a_negative_offset() {
    // BEQ -2 from $8000 lands back on the branch itself.
    let (cpu, cycles) = exec(&[0xF0, 0xFE], |cpu| set_status(cpu, Z));
    assert_eq!(cpu.pc, PROG, "offset is signed");
    assert_eq!(cycles, 3);
}

#[test]
fn a_branch_offset_is_relative_to_the_following_instruction() {
    // An offset of 0 must land on the instruction after the branch.
    let (cpu, _) = exec(&[0xF0, 0x00], |cpu| set_status(cpu, Z));
    assert_eq!(cpu.pc, PROG + 2);
}

#[test]
fn jmp_absolute_sets_the_program_counter() {
    let (cpu, cycles) = exec_bare(&[0x4C, 0x34, 0x12]);
    assert_eq!(cpu.pc, 0x1234);
    assert_eq!(cycles, 3);
}

#[test]
fn jsr_pushes_the_address_of_its_last_byte() {
    // JSR $1234 at $8000. The 6502 pushes PC-1 of the return point, i.e. $8002.
    let (mut cpu, cycles) = exec_bare(&[0x20, 0x34, 0x12]);

    assert_eq!(cpu.pc, 0x1234);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.sp, SP_RESET - 2, "JSR pushes two bytes");
    assert_eq!(peek(&mut cpu, 0x01FD), 0x80, "high byte of $8002");
    assert_eq!(peek(&mut cpu, 0x01FC), 0x02, "low byte of $8002");
}

#[test]
fn rts_returns_to_the_pushed_address_plus_one() {
    let (cpu, cycles) = exec(&[0x60], |cpu| {
        cpu.sp = 0xFB;
        poke(cpu, 0x01FC, &[0x02, 0x80]); // low then high of $8002
    });
    assert_eq!(cpu.pc, 0x8003, "RTS adds one to the pulled address");
    assert_eq!(cpu.sp, 0xFD);
    assert_eq!(cycles, 6);
}

#[test]
fn jsr_and_rts_round_trip() {
    let mut cpu = cpu();
    poke(&mut cpu, PROG, &[0x20, 0x00, 0x90]); // JSR $9000
    poke(&mut cpu, 0x9000, &[0x60]); // RTS

    step(&mut cpu);
    assert_eq!(cpu.pc, 0x9000);
    step(&mut cpu);
    assert_eq!(cpu.pc, PROG + 3, "RTS must land after the JSR");
    assert_eq!(cpu.sp, SP_RESET, "stack must be balanced");
}

#[test]
fn rti_restores_the_flags_and_program_counter() {
    let (cpu, cycles) = exec(&[0x40], |cpu| {
        cpu.sp = 0xFB;
        set_status(cpu, 0);
        // status, then low, then high.
        poke(cpu, 0x01FC, &[status_byte(C | V), 0x34, 0x12]);
    });

    assert_eq!(cpu.pc, 0x1234, "RTI does not add one");
    assert_eq!(cpu.sp, 0xFE);
    assert!(cpu.status.carry, "C restored from the stack");
    assert!(cpu.status.overflow, "V restored from the stack");
    assert_eq!(cycles, 6);
}

#[test]
fn rti_restores_every_real_flag() {
    let (cpu, _) = exec(&[0x40], |cpu| {
        cpu.sp = 0xFB;
        set_status(cpu, 0);
        poke(
            cpu,
            0x01FC,
            &[status_byte(N | V | D | I | Z | C), 0x00, 0x90],
        );
    });
    assert!(cpu.status.negative);
    assert!(cpu.status.overflow);
    assert!(cpu.status.decimal);
    assert!(cpu.status.interrupt);
    assert!(cpu.status.zero);
    assert!(cpu.status.carry);
}

#[test]
fn brk_pushes_state_and_vectors_through_fffe() {
    // BRK is a two-byte instruction: it pushes the address *after* the padding
    // byte, pushes the status with B set, sets I, and jumps to ($FFFE).
    let (mut cpu, cycles) = exec(&[0x00, 0xEA], |cpu| {
        set_status(cpu, 0);
        poke(cpu, 0xFFFE, &[0x34, 0x12]);
    });

    assert_eq!(cycles, 7);
    assert_eq!(cpu.pc, 0x1234, "BRK must vector through $FFFE");
    assert_eq!(cpu.sp, SP_RESET - 3, "BRK pushes PCH, PCL and P");
    assert!(cpu.status.interrupt, "BRK sets the interrupt disable flag");

    assert_eq!(peek(&mut cpu, 0x01FD), 0x80, "high byte of $8002");
    assert_eq!(peek(&mut cpu, 0x01FC), 0x02, "low byte of $8002");

    let pushed = peek(&mut cpu, 0x01FB);
    assert_eq!(pushed & B, B, "BRK pushes P with the B flag set");
    assert_eq!(pushed & U, U, "bit 5 is always set when pushed");
}
