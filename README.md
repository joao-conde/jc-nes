# Nintendo Entertainment System (NES) emulator

Nintendo Entertainment System (NES) emulator written in Rust.


<p align="center">
  <img width="40%" height="40%" align="center" src="https://drive.google.com/uc?export=view&id=165-0HS7UfhjN3sHJG7l3kGcDYuPH-QQE">
  <img width="40%" height="40%" align="center" src="https://drive.google.com/uc?export=view&id=1UpJ0S0gQ-Ybjt4UOFihcRkUtUimmE0J3">
</p>

<p align="center">
  <img width="40%" height="40%" align="center" src="https://drive.google.com/uc?export=view&id=1Jbl0HxsG46ijFEdCGPEi5WeebvkpZ8PA">
  <img width="40%" height="40%" align="center" src="https://drive.google.com/uc?export=view&id=1q-iU_ODlkV9vbK6A7YPtifDpRVPCGISE">
</p>

## Repository Contents

`core` contains the Rust emulator crate.

`frontends` contains multiple usages of this crate like:
- `frontends/desktop`: a desktop NES emulator
- `frontends/web`: a browser NES emulator powered by WebAssembly
- `frontends/mobile`: a mobile NES emulator powered by Android's NDK

# Running
## Desktop Application (with SDL)

```
$ cd frontends/desktop/
$ cargo run --release <ROM PATH>
```
