#![allow(dead_code)]

mod addressing;
mod arithmetic;
mod branch_jump;
mod harte;
mod illegal;
mod interrupts;
mod load_store;
mod logic;
mod stack_flags;
mod timing;

use crate::bus::{Bus, Device};
use crate::cpu::status::Status;
use crate::cpu::Cpu;
use crate::ram::Ram;

/// Address test programs are assembled at.
pub(super) const PROG: u16 = 0x8000;

/// Stack pointer the hardware settles on after reset.
pub(super) const SP_RESET: u8 = 0xFD;

// Status bit masks, in hardware bit order.
pub(super) const C: u8 = 0x01;
pub(super) const Z: u8 = 0x02;
pub(super) const I: u8 = 0x04;
pub(super) const D: u8 = 0x08;
pub(super) const B: u8 = 0x10;
pub(super) const U: u8 = 0x20;
pub(super) const V: u8 = 0x40;
pub(super) const N: u8 = 0x80;

/// A CPU wired to a flat, uniquely-mapped 64 KB RAM.
pub(super) fn cpu() -> Cpu {
    let mut bus = Bus::default();
    bus.connect(0x0000..=0xFFFF, Ram::new(vec![0u8; 0x1_0000]));

    let mut cpu = Cpu::new(bus);
    cpu.sp = SP_RESET;
    cpu.pc = PROG;
    cpu
}

/// Write `bytes` starting at `addr`.
pub(super) fn poke(cpu: &mut Cpu, addr: u16, bytes: &[u8]) {
    for (i, byte) in bytes.iter().enumerate() {
        cpu.bus.write(addr.wrapping_add(i as u16), *byte);
    }
}

/// Read a single byte.
pub(super) fn peek(cpu: &mut Cpu, addr: u16) -> u8 {
    cpu.bus.read(addr)
}

/// Execute exactly one instruction at `cpu.pc` and return the cycles it claimed.
pub(super) fn step(cpu: &mut Cpu) -> u8 {
    cpu.cycle = 0;
    let opcode = cpu.bus.read(cpu.pc);
    cpu.process_opcode(opcode);
    cpu.cycle
}

/// Assemble `prog` at [`PROG`], apply `setup`, then execute one instruction.
pub(super) fn exec(prog: &[u8], setup: impl FnOnce(&mut Cpu)) -> (Cpu, u8) {
    let mut cpu = cpu();
    poke(&mut cpu, PROG, prog);
    setup(&mut cpu);
    let cycles = step(&mut cpu);
    (cpu, cycles)
}

/// Assemble `prog` at [`PROG`] and execute it with the default reset state.
pub(super) fn exec_bare(prog: &[u8]) -> (Cpu, u8) {
    exec(prog, |_| {})
}

/// Push a byte using the same convention as the CPU, for seeding RTS/RTI/PLA.
pub(super) fn push(cpu: &mut Cpu, val: u8) {
    let sp = cpu.sp;
    poke(cpu, 0x0100 + sp as u16, &[val]);
    cpu.sp = sp.wrapping_sub(1);
}

/// Read the byte the CPU most recently pushed.
pub(super) fn stack_top(cpu: &mut Cpu, depth: u16) -> u8 {
    let addr = 0x0100u16.wrapping_add(cpu.sp.wrapping_add(depth as u8) as u16);
    peek(cpu, addr)
}

#[track_caller]
pub(super) fn assert_nz(cpu: &Cpu, negative: bool, zero: bool) {
    assert_eq!(cpu.status.negative, negative, "N flag");
    assert_eq!(cpu.status.zero, zero, "Z flag");
}

#[track_caller]
pub(super) fn assert_nzc(cpu: &Cpu, negative: bool, zero: bool, carry: bool) {
    assert_nz(cpu, negative, zero);
    assert_eq!(cpu.status.carry, carry, "C flag");
}

/// Assemble a status byte the way the hardware stores it: bit 5 always set.
pub(super) fn status_byte(bits: u8) -> u8 {
    bits | 0x20
}

pub(super) fn set_status(cpu: &mut Cpu, bits: u8) {
    cpu.status = Status::from(status_byte(bits));
}

#[test]
fn flat_ram_is_readable_and_writable_everywhere() {
    let mut cpu = cpu();
    for addr in [0x0000u16, 0x00FF, 0x0100, 0x1FFF, 0x8000, 0xFFFC, 0xFFFF] {
        poke(&mut cpu, addr, &[0xA5]);
        assert_eq!(peek(&mut cpu, addr), 0xA5, "address {addr:#06X}");
    }
}

#[test]
fn step_reports_the_instructions_cycle_count() {
    // NOP is 2 cycles and touches nothing but PC.
    let (cpu, cycles) = exec_bare(&[0xEA]);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.pc, PROG + 1);
}

#[test]
fn step_reports_zero_for_an_unimplemented_opcode() {
    // 0x02 is JAM/KIL: not in the opcode table, so it must fall through to
    // the catch-all arm without claiming any cycles.
    let (_, cycles) = exec_bare(&[0x02]);
    assert_eq!(cycles, 0);
}

#[test]
fn status_round_trips_through_a_byte() {
    let bits = status_byte(N | V | D | C);
    let status = Status::from(bits);
    assert_eq!(u8::from(status), bits);
}
