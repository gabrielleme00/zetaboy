use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum JoypadButton {
    Right,
    Left,
    Up,
    Down,
    A,
    B,
    Select,
    Start,
}

impl JoypadButton {
    pub fn as_bit_index(&self) -> usize {
        match self {
            JoypadButton::Right => 0,
            JoypadButton::Left => 1,
            JoypadButton::Up => 2,
            JoypadButton::Down => 3,
            JoypadButton::A => 4,
            JoypadButton::B => 5,
            JoypadButton::Select => 6,
            JoypadButton::Start => 7,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Joypad {
    button_state: u8, // 8 bits for 8 buttons (0 = pressed, 1 = unpressed)
    select: u8,       // P1 register value (which button group is selected)
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            button_state: 0xFF, // All buttons unpressed
            select: 0xFF,       // Nothing selected initially
        }
    }

    /// Set the state of a button
    /// Returns true if a joypad interrupt should be requested (button press event)
    pub fn set_button_state(&mut self, button: JoypadButton, pressed: bool) -> bool {
        let bit_index = button.as_bit_index();

        let current_state = (self.button_state >> bit_index) & 1;
        let new_state = if pressed { 0 } else { 1 }; // 0 for pressed, 1 for unpressed

        let should_interrupt = current_state == 1 && new_state == 0;

        if pressed {
            self.button_state &= !(1 << bit_index); // Set bit to 0 (pressed)
        } else {
            self.button_state |= 1 << bit_index; // Set bit to 1 (unpressed)
        }

        should_interrupt
    }

    /// Read the P1 register (0xFF00)
    pub fn read_p1(&self) -> u8 {
        let mut result = self.select;

        // If P14 (Direction keys) is selected (0)
        if result & 0x10 == 0 {
            result = (result & 0xF0) | (self.button_state & 0x0F); // Use bits 0-3 for directions
        }
        // If P15 (Action keys) is selected (0)
        if result & 0x20 == 0 {
            result = (result & 0xF0) | ((self.button_state >> 4) & 0x0F); // Use bits 4-7 for actions
        }

        result
    }

    /// Write to the P1 register (0xFF00)
    /// Only bits 4-5 are writable (button group selection)
    pub fn write_p1(&mut self, value: u8) {
        self.select = (value & 0x30) | 0xCF;
    }
}
