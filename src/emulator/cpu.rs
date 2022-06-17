pub struct CPU {}

impl CPU {
    pub fn new() -> Self {
        Self {}
    }

    pub fn step(&self) -> Result<(), &'static str> {
        Err("CPU not implemented yet.")
    }
}