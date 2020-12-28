# (Data) Bus

How I am gonna do it in Rust:
- create bus object that allows one device to broadcast messages to subscribed devices
- devices only react if message within range
- bus struct has to open a multi-producer multi-consumer channel (mpmc)
  - this does not exist in rust, only mpsc
  - must create an abstraction before
- a device subs to the bus
- a device can broadcast a message
