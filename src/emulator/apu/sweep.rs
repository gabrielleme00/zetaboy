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
        }
    }

    pub fn trigger(&mut self, current_frequency: u16) {
        self.shadow_frequency = current_frequency;
        self.period_counter = self.configured_period;
        
        // Sweep is enabled if period or shift is non-zero
        self.enabled = self.configured_period != 0 || self.configured_shift != 0;
        
        // If shift is non-zero, perform initial frequency calculation
        if self.configured_shift != 0 {
            let _ = self.calculate_new_frequency();
        }
    }

    pub fn clock(&mut self) -> Option<u16> {
        if !self.enabled {
            return None;
        }

        if self.period_counter > 0 {
            self.period_counter -= 1;
        } else {
            self.period_counter = self.configured_period;
            
            if self.configured_period != 0 {
                let new_frequency = self.calculate_new_frequency();
                if let Some(freq) = new_frequency {
                    self.shadow_frequency = freq;
                    
                    // Perform overflow check again
                    if self.calculate_new_frequency().is_none() {
                        self.enabled = false;
                        return None;
                    }
                    
                    return Some(freq);
                }
            }
        }
        
        None
    }

    fn calculate_new_frequency(&self) -> Option<u16> {
        use SweepDirection::*;

        let offset = self.shadow_frequency >> self.configured_shift;
        
        let new_frequency = match self.configured_direction {
            Increase => self.shadow_frequency.wrapping_add(offset),
            Decrease => self.shadow_frequency.wrapping_sub(offset),
        };
        
        // Check for overflow (frequency > 2047)
        if new_frequency > 2047 {
            None
        } else {
            Some(new_frequency)
        }
    }
}

impl Default for Sweep {
    fn default() -> Self {
        Self::new()
    }
}