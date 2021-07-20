# Nintendo Entertainment System (NES) emulator

Nintendo Entertainment System (NES) emulator written in Rust.

<p align="center">
  <img width="40%" height="40%" align="center" src="https://drive.google.com/uc?export=view&id=1TIk4afXnPGvEJpSsquIfG0Y_VuTPDeMl">
  <img width="40%" height="40%" align="center" src="https://drive.google.com/uc?export=view&id=1UpJ0S0gQ-Ybjt4UOFihcRkUtUimmE0J3">
</p>

<p align="center">
  <img width="40%" height="40%" align="center" src="https://drive.google.com/uc?export=view&id=1Jbl0HxsG46ijFEdCGPEi5WeebvkpZ8PA">
  <img width="40%" height="40%" align="center" src="https://drive.google.com/uc?export=view&id=1q-iU_ODlkV9vbK6A7YPtifDpRVPCGISE">
</p>

# Running

## Desktop Application

```
$ cd examples/desktop/
$ cargo run --release <ROM PATH>
```

# Contributing

Each game uses a specific mapper, and there are a ton of them. I will not implement them all. Incoming PRs with new mapper implementations are welcome. Check the [mappers module](./src/cartridge/mappers) to see the current implementation and the [NESDev Wiki](https://wiki.nesdev.com/w/index.php/Mapper) for more information.
