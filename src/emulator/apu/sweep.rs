use super::*;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SweepDirection {
    Increase,
    Decrease,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Sweep {
    pub enabled: bool,
    pub configured_period: u8,
    pub configured_direction: SweepDirection,
    pub configured_shift: u8,
    period_counter: u8,
    shadow_frequency: u16,
    sweep_length_addend: u16,
    overflow_occurred: bool,
}

impl Sweep {
    pub fn new() -> Self {
        Self {
            enabled: false,
            configured_period: 0,
            configured_direction: SweepDirection::Increase,
            configured_shift: 0,
            period_counter: 0,
            shadow_frequency: 0,
            sweep_length_addend: 0,
            overflow_occurred: false,
        }
    }

    /// Trigger the sweep unit with the current frequency.
    /// Returns true if overflow occurred and the channel should be disabled.
    pub fn trigger(&mut self, current_frequency: u16) -> bool {
        self.shadow_frequency = current_frequency;
        self.sweep_length_addend = current_frequency >> self.configured_shift;

        // Reload period counter (treating 0 as 8)
        self.period_counter = if self.configured_period == 0 {
            8
        } else {
            self.configured_period
        };

        self.overflow_occurred = false;

        // Sweep is enabled if period or shift is non-zero
        self.enabled = self.configured_period != 0 || self.configured_shift != 0;

        // If shift is non-zero, perform initial frequency calculation and check for overflow
        if self.configured_shift != 0 {
            let new_frequency = self.calculate_new_frequency();
            if new_frequency.is_none() {
                self.enabled = false;
                self.overflow_occurred = true;
                return true; // Overflow - channel should be disabled
            }
        }

        false // No overflow
    }

    pub fn clock(&mut self) -> Option<u16> {
        if !self.enabled {
            return None;
        }

        if self.period_counter > 0 {
            self.period_counter -= 1;
        }

        if self.period_counter == 0 {
            // Reload period counter (treating 0 as 8)
            self.period_counter = if self.configured_period == 0 {
                8
            } else {
                self.configured_period
            };

            // Only perform frequency calculation if configured period is non-zero
            if self.configured_period != 0 && self.configured_shift != 0 {
                let new_frequency = self.calculate_new_frequency();
                if let Some(freq) = new_frequency {
                    // Write new frequency to shadow register and update addend
                    self.shadow_frequency = freq;
                    self.sweep_length_addend = freq >> self.configured_shift;

                    // Perform overflow check again immediately
                    if self.calculate_new_frequency().is_none() {
                        self.enabled = false;
                        self.overflow_occurred = true;
                        return None;
                    }

                    // Return the new frequency to update NR13/NR14
                    return Some(freq);
                } else {
                    // Overflow on first calculation
                    self.enabled = false;
                    self.overflow_occurred = true;
                    return None;
                }
            }
        }

        None
    }

    pub fn should_disable_channel(&self) -> bool {
        self.overflow_occurred
    }

    fn calculate_new_frequency(&self) -> Option<u16> {
        use SweepDirection::*;

        let new_frequency = match self.configured_direction {
            Increase => {
                let freq = self.shadow_frequency + self.sweep_length_addend;
                if freq > 2047 {
                    return None;
                }
                freq
            }
            Decrease => {
                if self.configured_shift > 0 {
                    self.shadow_frequency
                        .saturating_sub(self.sweep_length_addend)
                } else {
                    self.shadow_frequency
                }
            }
        };

        Some(new_frequency)
    }
}

impl Default for Sweep {
    fn default() -> Self {
        Self::new()
    }
}
