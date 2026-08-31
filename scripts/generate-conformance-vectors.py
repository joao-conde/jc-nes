#!/usr/bin/env python3
"""Regenerate the vendored conformance sample.

Reads the full reference test data fetched by fetch-conformance-tests.sh and
emits a fixed sample of it as Rust source, so the conformance tests run with no
setup and no network. Deterministic: takes the first N cases of each opcode file.

    python3 scripts/generate-conformance-vectors.py [N]
"""
import json
import os
import sys

N = int(sys.argv[1]) if len(sys.argv) > 1 else 25
ROOT = os.path.join(os.path.dirname(__file__), "..", "jc-nes")
SRC = os.path.join(ROOT, ".conformance", "v1")
DST = os.path.join(ROOT, "src", "cpu", "tests", "conformance", "vectors.rs")

HEADER = '''//! Vendored sample of the reference test data. Generated - do not edit.
//!
//! Regenerate with `python3 scripts/generate-conformance-vectors.py`.
//!
//! {n} cases per opcode, {total} in total, taken from the data described in the
//! parent module. This sample is committed so the conformance tests always run:
//! a test that silently skips when its data is missing reports as passing, which
//! is worse than no test at all. The full {full} cases remain available through
//! the ignored test in the parent module.

/// Register state before or after an instruction.
pub(super) struct State {{
    pub pc: u16,
    pub s: u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: u8,
}}

pub(super) struct Case {{
    pub opcode: u8,
    pub initial: State,
    /// Memory seeded before execution.
    pub ram: &'static [(u16, u8)],
    pub expected: State,
    /// Only the addresses the instruction changed; everything else in `ram` must
    /// still hold its seeded value afterwards.
    pub writes: &'static [(u16, u8)],
    pub cycles: u8,
}}

#[allow(clippy::too_many_arguments)]
const fn c(
    opcode: u8,
    pc: u16,
    s: u8,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    ram: &'static [(u16, u8)],
    fpc: u16,
    fs: u8,
    fa: u8,
    fx: u8,
    fy: u8,
    fp: u8,
    writes: &'static [(u16, u8)],
    cycles: u8,
) -> Case {{
    Case {{
        opcode,
        initial: State {{ pc, s, a, x, y, p }},
        ram,
        expected: State {{ pc: fpc, s: fs, a: fa, x: fx, y: fy, p: fp }},
        writes,
        cycles,
    }}
}}

#[rustfmt::skip]
pub(super) static CASES: &[Case] = &[
'''


def main():
    if not os.path.isdir(SRC):
        sys.exit(f"no reference data at {SRC}; run scripts/fetch-conformance-tests.sh first")

    rows, total = [], 0
    for op in range(256):
        with open(os.path.join(SRC, f"{op:02x}.json")) as fh:
            for case in json.load(fh)[:N]:
                i, f = case["initial"], case["final"]
                seeded = dict((a, v) for a, v in i["ram"])
                final = dict((a, v) for a, v in f["ram"])
                writes = sorted((a, v) for a, v in final.items() if seeded.get(a) != v)
                fmt = lambda pairs: ",".join(f"(0x{a:04X},0x{v:02X})" for a, v in pairs)
                rows.append(
                    f"    c(0x{op:02X},"
                    f"0x{i['pc']:04X},0x{i['s']:02X},0x{i['a']:02X},0x{i['x']:02X},"
                    f"0x{i['y']:02X},0x{i['p']:02X},&[{fmt(sorted(seeded.items()))}],"
                    f"0x{f['pc']:04X},0x{f['s']:02X},0x{f['a']:02X},0x{f['x']:02X},"
                    f"0x{f['y']:02X},0x{f['p']:02X},&[{fmt(writes)}],{len(case['cycles'])}),"
                )
                total += 1

    os.makedirs(os.path.dirname(DST), exist_ok=True)
    with open(DST, "w", newline="\n") as fh:
        fh.write(HEADER.format(n=N, total=total, full=256 * 10000))
        fh.write("\n".join(rows))
        fh.write("\n];\n")
    print(f"wrote {total} cases to {DST}")


if __name__ == "__main__":
    main()
