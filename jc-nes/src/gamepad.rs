use crate::bus::Device;

#[derive(Default)]
pub struct Gamepad {
    state: u8,
    state_snapshot: u8,
}

pub enum Button {
    Right,
    Left,
    Down,
    Up,
    Start,
    Select,
    B,
    A,
}

impl Device for Gamepad {
    fn read(&mut self, _address: u16) -> u8 {
        let data = (self.state_snapshot & 0x80) >> 7;
        self.state_snapshot <<= 1;
        data
    }

    fn write(&mut self, _address: u16, _data: u8) {
        self.state_snapshot = self.state;
    }
}

impl Gamepad {
    pub fn btn_down(&mut self, btn: Button) {
        match btn {
            Button::Right => self.state |= 0x01,
            Button::Left => self.state |= 0x02,
            Button::Down => self.state |= 0x04,
            Button::Up => self.state |= 0x08,
            Button::Start => self.state |= 0x10,
            Button::Select => self.state |= 0x20,
            Button::B => self.state |= 0x40,
            Button::A => self.state |= 0x80,
        }
    }

    pub fn btn_up(&mut self, btn: Button) {
        match btn {
            Button::Right => self.state &= !0x01,
            Button::Left => self.state &= !0x02,
            Button::Down => self.state &= !0x04,
            Button::Up => self.state &= !0x08,
            Button::Start => self.state &= !0x10,
            Button::Select => self.state &= !0x20,
            Button::B => self.state &= !0x40,
            Button::A => self.state &= !0x80,
        }
    }
}
