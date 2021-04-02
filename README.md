# Nintendo Entertainment System (NES) emulator

Nintendo Entertainment System (NES) emulator written in Rust.

`core` contains the Rust emulator crate.

`frontends` contains multiple usages of this crate like:
- `frontends/desktop`: a desktop NES emulator

# Running
## Desktop Application (with SDL)

```
$ cd frontends/desktop/
$ cargo run --release <ROM PATH>
```
