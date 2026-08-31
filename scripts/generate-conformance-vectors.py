#!/usr/bin/env python3
"""Regenerate the vendored conformance sample as readable Rust.

Reads the reference data fetched by fetch-conformance-tests.sh and emits a fixed
sample as Rust source, so the conformance tests run with no setup and no network.
Deterministic: takes the first N cases of each opcode file.

Opcode names and addressing modes are parsed out of the emulator's own dispatch
table in src/cpu/mod.rs, so they cannot drift from the implementation. They label
the data; they are not part of any assertion.

    python3 scripts/generate-conformance-vectors.py [N]
"""
import json
import os
import re
import sys

N = int(sys.argv[1]) if len(sys.argv) > 1 else 25
ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "jc-nes")
DATA = os.path.join(ROOT, ".conformance", "v1")
DISPATCH = os.path.join(ROOT, "src", "cpu", "mod.rs")
DST = os.path.join(ROOT, "src", "cpu", "tests", "conformance", "vectors.rs")

# Instruction helpers that share one mnemonic.
MNEMONIC = {
    "asl_acc": "ASL", "asl_mem": "ASL", "lsr_acc": "LSR", "lsr_mem": "LSR",
    "rol_acc": "ROL", "rol_mem": "ROL", "ror_acc": "ROR", "ror_mem": "ROR",
    "nop_unoff": "NOP",
}
# The unstable stores dispatch through `abs`/`indy_base` and index internally,
# so the dispatch mode understates their real addressing.
REAL_MODE = {0x93: "indy", 0x9B: "absy", 0x9C: "absx", 0x9E: "absy", 0x9F: "absy"}
MODE_SUFFIX = {
    "imp": "IMP", "acc": "ACC", "imm": "IMM", "zp": "ZP", "zpx": "ZPX", "zpy": "ZPY",
    "abs": "ABS", "absx": "ABSX", "absy": "ABSY", "ind": "IND", "indx": "INDX",
    "indy": "INDY", "indy_base": "INDY", "relative": "REL",
}


def dispatch_table():
    """opcode -> (instruction fn, addressing mode), read from the emulator source."""
    src = open(DISPATCH, encoding="utf-8").read()
    body = src[src.index("fn process_opcode"): src.index("fn execute<T>")]
    flat = re.sub(r"\s+", " ", body)
    table = {}
    executed = r"((?:0x[0-9A-F]{2} \| )*0x[0-9A-F]{2}) => (?:\{ )?self\.execute\(Cpu::(\w+), Cpu::(\w+),"
    for m in re.finditer(executed, flat):
        for op in re.findall(r"0x([0-9A-F]{2})", m.group(1)):
            table[int(op, 16)] = (m.group(3), m.group(2))
    jam = r"((?:0x[0-9A-F]{2} \| )*0x[0-9A-F]{2}) => \{ eprintln!.*?self\.jammed = true"
    for m in re.finditer(jam, flat):
        for op in re.findall(r"0x([0-9A-F]{2})", m.group(1)):
            table[int(op, 16)] = ("jam", "imp")
    if len(table) != 256:
        sys.exit("parsed %d/256 opcodes from %s" % (len(table), DISPATCH))
    return table


def const_names(table):
    """A unique, readable Rust constant name per opcode."""
    base = {}
    for op, (fn, mode) in table.items():
        mnemonic = MNEMONIC.get(fn, fn.upper())
        base[op] = "%s_%s" % (mnemonic, MODE_SUFFIX[REAL_MODE.get(op, mode)])
    counts = {}
    for name in base.values():
        counts[name] = counts.get(name, 0) + 1
    # Disambiguate shared names (the unofficial NOPs, JAM, USBC) with the opcode.
    return {
        op: (name if counts[name] == 1 else "%s_%02X" % (name, op))
        for op, name in base.items()
    }


def disassemble(op, table, pc, b1, b2):
    fn, mode = table[op]
    mnemonic = MNEMONIC.get(fn, fn.upper())
    mode = REAL_MODE.get(op, mode)
    word = (b2 << 8) | b1
    offset = b1 - 256 if b1 > 127 else b1
    forms = {
        "imp": mnemonic,
        "acc": "%s A" % mnemonic,
        "imm": "%s #$%02X" % (mnemonic, b1),
        "zp": "%s $%02X" % (mnemonic, b1),
        "zpx": "%s $%02X,X" % (mnemonic, b1),
        "zpy": "%s $%02X,Y" % (mnemonic, b1),
        "abs": "%s $%04X" % (mnemonic, word),
        "absx": "%s $%04X,X" % (mnemonic, word),
        "absy": "%s $%04X,Y" % (mnemonic, word),
        "ind": "%s ($%04X)" % (mnemonic, word),
        "indx": "%s ($%02X,X)" % (mnemonic, b1),
        "indy": "%s ($%02X),Y" % (mnemonic, b1),
        "indy_base": "%s ($%02X),Y" % (mnemonic, b1),
        "relative": "%s $%04X" % (mnemonic, (pc + 2 + offset) & 0xFFFF),
    }
    return forms[mode]


def flags(p):
    """Status byte in the conventional NV-BDIZC form; set flags uppercase."""
    out = []
    for index, name in enumerate("NV-BDIZC"):
        on = (p >> (7 - index)) & 1
        if name == "-":
            out.append("1" if on else "0")
        else:
            out.append(name if on else name.lower())
    return "".join(out)


def pairs(items):
    return ", ".join("(0x%04X, 0x%02X)" % (a, v) for a, v in items)


def main():
    if not os.path.isdir(DATA):
        sys.exit("no reference data at %s; run scripts/fetch-conformance-tests.sh first" % DATA)
    table = dispatch_table()
    names = const_names(table)

    body = []
    total = 0
    for op in range(256):
        with open(os.path.join(DATA, "%02x.json" % op)) as fh:
            cases = json.load(fh)[:N]
        body.append("\n    // ---------- $%02X  %s ----------" % (op, names[op]))
        for case in cases:
            i = case["initial"]
            f = case["final"]
            seeded = dict((a, v) for a, v in i["ram"])
            final = dict((a, v) for a, v in f["ram"])
            writes = sorted((a, v) for a, v in final.items() if seeded.get(a) != v)
            b1 = seeded.get((i["pc"] + 1) & 0xFFFF, 0)
            b2 = seeded.get((i["pc"] + 2) & 0xFFFF, 0)
            body.append(
                "    Case {\n"
                "        opcode: %s,\n"
                '        asm: "%s",\n'
                "        initial: State { pc: 0x%04X, s: 0x%02X, a: 0x%02X, x: 0x%02X, y: 0x%02X, p: 0x%02X }, // %s\n"
                "        ram: &[%s],\n"
                "        expected: State { pc: 0x%04X, s: 0x%02X, a: 0x%02X, x: 0x%02X, y: 0x%02X, p: 0x%02X }, // %s\n"
                "        writes: &[%s],\n"
                "        cycles: %d,\n"
                "    },"
                % (
                    names[op],
                    disassemble(op, table, i["pc"], b1, b2),
                    i["pc"], i["s"], i["a"], i["x"], i["y"], i["p"], flags(i["p"]),
                    pairs(sorted(seeded.items())),
                    f["pc"], f["s"], f["a"], f["x"], f["y"], f["p"], flags(f["p"]),
                    pairs(writes),
                    len(case["cycles"]),
                )
            )
            total += 1

    consts = "\n".join(
        "pub(super) const %s: u8 = 0x%02X;" % (names[op], op) for op in range(256)
    )
    header = HEADER % (N, total, 256 * 10000, consts)

    os.makedirs(os.path.dirname(DST), exist_ok=True)
    with open(DST, "w", newline="\n") as fh:
        fh.write(header)
        fh.write("\n".join(body))
        fh.write("\n];\n")
    print("wrote %d cases to %s" % (total, DST))


HEADER = """//! Vendored sample of the reference test data. Generated - do not edit.
//!
//! Regenerate with `python3 scripts/generate-conformance-vectors.py`.
//!
//! %d cases per opcode, %d in total, sampled from the data described in the
//! parent module. This sample is committed so the conformance tests always run:
//! a test that silently skips when its data is missing reports as passing, which
//! is worse than no test at all. The full %d cases stay available through the
//! ignored tests in the parent module.
//!
//! Each case is one instruction executed once. `ram` is the memory seeded before
//! execution, starting with the instruction's own bytes at `initial.pc`. `writes`
//! lists only the addresses the instruction changed; every other seeded address
//! must still hold its original value afterwards. The trailing comment on each
//! `State` renders the status byte as `NV-BDIZC`, with set flags in uppercase.
//!
//! The opcode names below are parsed from the emulator's own dispatch table, so
//! they cannot drift from the implementation. They label the data; the assertions
//! use only the numbers.

/// Register state before or after an instruction.
pub(super) struct State {
    pub pc: u16,
    pub s: u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: u8,
}

pub(super) struct Case {
    pub opcode: u8,
    /// Disassembly of the instruction under test, so failures name it.
    pub asm: &'static str,
    pub initial: State,
    /// Memory seeded before execution, the instruction's own bytes included.
    pub ram: &'static [(u16, u8)],
    pub expected: State,
    /// Only the addresses the instruction changed.
    pub writes: &'static [(u16, u8)],
    pub cycles: u8,
}

%s

#[rustfmt::skip]
pub(super) static CASES: &[Case] = &[
"""


if __name__ == "__main__":
    main()
