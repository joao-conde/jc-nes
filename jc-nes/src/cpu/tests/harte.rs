//! Verification against the SingleStepTests `nes6502` corpus.
//!
//! Source: <https://github.com/SingleStepTests/65x02>, directory `nes6502/v1`.
//! 10,000 machine-generated cases per opcode, 2,560,000 in total, captured from
//! real 6502 behaviour. The `nes6502` variant is the 2A03 flavour, which ignores
//! the decimal flag.
//!
//! Every expectation here comes from that corpus verbatim. Nothing in this file
//! is hand-authored: the harness loads a case's `initial` state, runs one
//! instruction, and compares against the case's `final` state.
//!
//! Fetch the corpus with `scripts/fetch-harte.sh`. It is gitignored (1.1 GB), and
//! these tests report themselves as skipped when it is absent.
//!
//! # What is checked
//!
//! The complete post-instruction state - `PC`, `S`, `A`, `X`, `Y`, `P`, and every
//! memory address the case names - plus the total cycle count, which the corpus
//! gives as `cycles.len()`.
//!
//! # What is not checked
//!
//! The cycle-by-cycle bus trace in `cycles`. jc-nes executes a whole instruction
//! in one step and then reports its cost, so it has no per-cycle bus activity to
//! compare. The address/value/direction of each individual bus access therefore
//! goes unverified; only the totals do.

use super::*;
use serde::de::IgnoredAny;
use serde::Deserialize;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct Case {
    name: String,
    initial: State,
    #[serde(rename = "final")]
    expected: State,
    /// `[address, value, "read" | "write"]` per cycle. Only the length is used;
    /// see the module docs.
    cycles: Vec<(u16, u8, IgnoredAny)>,
}

#[derive(Deserialize)]
struct State {
    pc: u16,
    s: u8,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    ram: Vec<(u16, u8)>,
}

/// Location of the corpus, overridable with `JC_NES_HARTE_DIR`.
fn corpus_dir() -> Option<PathBuf> {
    let dir = match std::env::var("JC_NES_HARTE_DIR") {
        Ok(path) => PathBuf::from(path),
        Err(_) => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".harte/v1"),
    };
    if dir.is_dir() {
        Some(dir)
    } else {
        None
    }
}

/// Load a case's initial state, run one instruction, and diff against `final`.
///
/// Returns `None` when the case passes, or a description of every mismatch.
fn run_case(cpu: &mut Cpu, case: &Case) -> Option<String> {
    cpu.pc = case.initial.pc;
    cpu.sp = case.initial.s;
    cpu.a = case.initial.a;
    cpu.x = case.initial.x;
    cpu.y = case.initial.y;
    cpu.status = Status::from(case.initial.p);
    for &(address, value) in &case.initial.ram {
        poke(cpu, address, &[value]);
    }

    // A panic is a failure of this case, not of the whole run.
    let cycles = match panic::catch_unwind(AssertUnwindSafe(|| step(cpu))) {
        Ok(cycles) => cycles,
        Err(_) => return Some("panicked".to_string()),
    };

    let mut diffs = Vec::new();
    let mut cmp = |what: &str, got: u16, want: u16| {
        if got != want {
            diffs.push(format!("{what}={got:#06X} want {want:#06X}"));
        }
    };
    cmp("PC", cpu.pc, case.expected.pc);
    cmp("S", cpu.sp as u16, case.expected.s as u16);
    cmp("A", cpu.a as u16, case.expected.a as u16);
    cmp("X", cpu.x as u16, case.expected.x as u16);
    cmp("Y", cpu.y as u16, case.expected.y as u16);
    cmp("P", u8::from(cpu.status) as u16, case.expected.p as u16);

    for &(address, value) in &case.expected.ram {
        let got = peek(cpu, address);
        if got != value {
            diffs.push(format!("[{address:#06X}]={got:#04X} want {value:#04X}"));
        }
    }

    if cycles as usize != case.cycles.len() {
        diffs.push(format!("cycles={} want {}", cycles, case.cycles.len()));
    }

    if diffs.is_empty() {
        None
    } else {
        Some(diffs.join(", "))
    }
}

/// Zero every address a case touched, so one CPU can be reused across cases.
///
/// The corpus names every location an instruction reads or writes across
/// `initial.ram`, `final.ram` and `cycles`, so clearing all three leaves no
/// residue for the next case.
fn clear_case(cpu: &mut Cpu, case: &Case) {
    for &(address, _) in &case.initial.ram {
        poke(cpu, address, &[0]);
    }
    for &(address, _) in &case.expected.ram {
        poke(cpu, address, &[0]);
    }
    for cycle in &case.cycles {
        poke(cpu, cycle.0, &[0]);
    }
}

struct Report {
    opcode: u8,
    failed: usize,
    total: usize,
    samples: Vec<String>,
}

fn check_opcode(dir: &Path, opcode: u8) -> Option<Report> {
    let path = dir.join(format!("{opcode:02x}.json"));
    let raw = std::fs::read_to_string(&path).ok()?;
    let cases: Vec<Case> = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("{} is not valid corpus JSON: {e}", path.display()));

    let mut cpu = cpu();
    let mut report = Report {
        opcode,
        failed: 0,
        total: cases.len(),
        samples: Vec::new(),
    };

    for case in &cases {
        if let Some(diff) = run_case(&mut cpu, case) {
            report.failed += 1;
            if report.samples.len() < 3 {
                report
                    .samples
                    .push(format!("  \"{}\": {}", case.name, diff));
            }
        }
        clear_case(&mut cpu, case);
    }

    Some(report)
}

/// Run every opcode whose high nibble is `high`.
fn check_nibble(high: u8) {
    let Some(dir) = corpus_dir() else {
        eprintln!(
            "SKIPPED: nes6502 corpus not found. Run scripts/fetch-harte.sh, \
             or set JC_NES_HARTE_DIR."
        );
        return;
    };

    // Unimplemented opcodes panic or stall on every case; keep the noise down.
    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let reports: Vec<Report> = (0..=0x0F)
        .filter_map(|low| check_opcode(&dir, (high << 4) | low))
        .collect();
    panic::set_hook(previous_hook);

    let failing: Vec<&Report> = reports.iter().filter(|r| r.failed > 0).collect();
    if failing.is_empty() {
        return;
    }

    let mut out = String::new();
    let total_failed: usize = failing.iter().map(|r| r.failed).sum();
    let total_cases: usize = reports.iter().map(|r| r.total).sum();
    out.push_str(&format!(
        "{total_failed} of {total_cases} corpus cases failed across {} opcode(s):\n",
        failing.len()
    ));
    for report in failing {
        out.push_str(&format!(
            "{:#04X}: {}/{} failed\n{}\n",
            report.opcode,
            report.failed,
            report.total,
            report.samples.join("\n")
        ));
    }
    panic!("{}", out);
}

macro_rules! nibble_tests {
    ($($name:ident => $high:literal),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                check_nibble($high);
            }
        )*
    };
}

nibble_tests! {
    corpus_opcodes_0x => 0x0,
    corpus_opcodes_1x => 0x1,
    corpus_opcodes_2x => 0x2,
    corpus_opcodes_3x => 0x3,
    corpus_opcodes_4x => 0x4,
    corpus_opcodes_5x => 0x5,
    corpus_opcodes_6x => 0x6,
    corpus_opcodes_7x => 0x7,
    corpus_opcodes_8x => 0x8,
    corpus_opcodes_9x => 0x9,
    corpus_opcodes_ax => 0xA,
    corpus_opcodes_bx => 0xB,
    corpus_opcodes_cx => 0xC,
    corpus_opcodes_dx => 0xD,
    corpus_opcodes_ex => 0xE,
    corpus_opcodes_fx => 0xF,
}
