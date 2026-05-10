use crate::detectors::primatives::candles::candle::Candle;

pub enum SwingType{
    High,
    Low,
}
pub enum SwingMode {
    Body, 
    Wick,
}
pub enum Classification {
    Major, 
    Intermediate, 
    Minor,
}
pub enum ConfirmationType {
    PercentCandle(f64),
    PercentPrice(f64),
    Ticks(u64, f64),
    Candles(usize)
}



//Basic Swing Type
pub struct Swing{
    pub candle: Candle,
    pub swing_type: SwingType,
    pub timestamp: u64,
    pub dominance: usize,
    pub swing_classification: Classification,
}

impl Swing {
    fn new(candle: Candle, swing_type: SwingType, timestamp: u64 ) -> Self {
        Self{
            candle: candle,
            swing_type: swing_type,
            timestamp: timestamp,
            dominance: 1, 
            swing_classification: Classification::Minor, 

        }
    }
}

//Swing memory for long term look back
pub struct SwingMemory {
    pub swing_lookback: usize,
    pub detected_swings: Vec<Swing>,
}

impl SwingMemory {
    fn new( swing_lookback: usize, ) -> Self {
        Self {
            swing_lookback: swing_lookback,
            detected_swings: Vec::with_capacity(swing_lookback),
        }
    }
}

//detecting swings for streamed data
pub struct SwingConfirmParams {
    params: Option<ConfirmationType>
}
pub struct StremingSwingDetector {
    current_candle: Option<Candle>,
    current_candidate: Option<Candle>, 
    swing_mode: SwingMode, 
    seeking_mode: SwingType, 
    confirmation_mode: ConfirmationType,
    candle_counter: usize, 
    swing_lookback: usize,
    memory: SwingMemory,
}

impl StremingSwingDetector {
    pub fn new( swing_mode: SwingMode, seeking_mode: SwingType, confirmation_mode: ConfirmationType, swing_lookback: usize ) -> Self {
        Self {
            current_candle: None, 
            current_candidate: None,
            candle_counter: 0,
            swing_mode: swing_mode,
            seeking_mode: seeking_mode,
            confirmation_mode: confirmation_mode,
            swing_lookback: swing_lookback,
            memory: SwingMemory::new(swing_lookback)
        }
    }

    pub fn get_swings(&self) -> Vec<(u64, f64, bool)> {
        self.memory.detected_swings.iter().map(|s| {
            let is_high = matches!(s.swing_type, SwingType::High); 
            (s.timestamp, s.candle.high, is_high)
        }).collect()
    }

    fn clean_candles(&mut self) {
        self.current_candidate = None; 
        self.current_candle = None;
    }  

    pub fn process_candle(&mut self, incoming_candle: Candle) {
        if let Some(current_candle) = self.current_candle {
            if let Some(cur_candidate) = self.current_candidate {
                let (can_high, can_low) = self.get_candidate_levels(&cur_candidate);
                match self.seeking_mode {
                    SwingType::High => {
                        if current_candle.high > can_high {
                            // New higher high — replace candidate, keep looking
                            self.current_candidate = Some(current_candle);
                        } else if current_candle.close < can_low {
                            // Price closed below candidate low — check confirmation
                            self.swing_confirmation();
                            self.current_candidate = None;
                        }
                        // Otherwise just advance
                        self.current_candle = Some(incoming_candle);
                    }
                    SwingType::Low => {
                        if current_candle.low < can_low {
                            // New lower low — replace candidate, keep looking
                            self.current_candidate = Some(current_candle);
                        } else if current_candle.close > can_high {
                            // Price closed above candidate high — check confirmation
                            self.swing_confirmation();
                            self.current_candidate = None;
                        }
                        // Otherwise just advance
                        self.current_candle = Some(incoming_candle);
                    }
                }
            } else {
                self.current_candidate = Some(current_candle);
                self.current_candle = Some(incoming_candle);
            }
        } else {
            self.current_candle = Some(incoming_candle);
        }
    }

    fn get_candidate_levels(&self, candle: &Candle) -> (f64, f64){
        match self.swing_mode {
            SwingMode::Wick => (candle.high, candle.low),
            SwingMode::Body => {
                if candle.is_bullish() {
                    (candle.close, candle.open)
                } 
                else {
                    (candle.open, candle.close)
                }
            }
        }
    }

    fn percent_candle(&mut self, percent:f64) -> Option<Swing> {
        if let Some(candidate) = &self.current_candidate {
            let (high, low) = self.get_candidate_levels(candidate);
            let trigger = match self.seeking_mode {
                SwingType::High => low - (high - low) * percent, 
                SwingType::Low => high +  (high - low) * percent, 
            }; 

            self.swing_detected(trigger)
        } 
        else {
            None
        }
    }

    fn percent_price(&mut self, percent:f64) -> Option<Swing> {
        if let Some(candidate) = &self.current_candidate {
            let (high, low) = self.get_candidate_levels(candidate);
            let trigger = match self.seeking_mode {
                SwingType::High => low - (low * (1.0 - percent)),
                SwingType::Low => high + (high * (1.0 - percent)), 
            }; 
            self.swing_detected(trigger)
        } else {
            None
        }
    }

    fn ticks(&mut self, ticks: u64, tick_size: f64) -> Option<Swing> {
        if let Some(candidate) = &self.current_candidate {
            let (high, low) = self.get_candidate_levels(candidate);
            let trigger = match self.seeking_mode {
                SwingType::High => low - (ticks as f64 * tick_size),
                SwingType::Low => high + (ticks as f64 * tick_size),
            }; 
            self.swing_detected(trigger)
        } else {
            None
        }

    }

    fn candle_count(&mut self, count: usize) -> Option<Swing> {
        if let Some(candidate) = &self.current_candidate {
            let (high, low) = self.get_candidate_levels(candidate);
            if let Some(curr) = &self.current_candle {
                let trigger = match self.seeking_mode {
                    SwingType::High => {
                        
                        if self.candle_counter != 0 && low < curr.close {
                            self.candle_counter -= 1;
                            return None
                        }
                        else {
                            curr.close
                        }
                    }
                    SwingType::Low => {
                        if self.candle_counter != 0 && high > curr.close {
                            self.candle_counter -= 1;
                            return None
                        }
                        else {
                            
                            curr.close
                        }
                    }
                };
                if let Some(swing) = self.swing_detected(trigger) {
                    self.candle_counter = count;
                    Some(swing)

                }
                else {
                    None
                }
            } else {
                None
            }
            
        } else {
            None 
        }
    }

    fn swing_detected(&mut self, trigger: f64,) -> Option<Swing> {
        match self.seeking_mode {

            SwingType::High => {

                if let Some(curr) = &self.current_candle {
                    if curr.close <  trigger{
                        if let Some(candidate) = &self.current_candidate{
                            self.seeking_mode = SwingType::Low; 
                            Some(Swing::new(*candidate, SwingType::High, candidate.timestamp))
                        }
                        else {
                            None
                        }
                    }
                    else {
                        return None
                    }
                    
                }
                else {
                    return None
                }
            }
        
            SwingType::Low => {
                if let Some(curr) = &self.current_candle{
                    if curr.close > trigger {
                        if let Some(candidate) = &self.current_candidate {
                            self.seeking_mode = SwingType::High;
                            Some(Swing::new(*candidate, SwingType::Low, candidate.timestamp))
                        }
                        else {
                            None
                        }
                        
                    } 
                    else {
                        return None
                    } 
                }
                else {
                    return None
                }
            }

        }
        
        
    }

    fn swing_confirmation(&mut self){
        match self.confirmation_mode {
                            ConfirmationType::PercentCandle(percent) => {
                                if let Some(swing) = self.percent_candle(percent){
                                    self.memory.detected_swings.push(swing)
                                }
                                
                            },
                            ConfirmationType::PercentPrice(percent) => {
                                if let Some(swing) = self.percent_price(percent){
                                    self.memory.detected_swings.push(swing)

                                }
                                                            }
                            ConfirmationType::Ticks(ticks, tick_size) => {  
                                if let Some(swing) = self.ticks(ticks, tick_size){
                                    self.memory.detected_swings.push(swing)
                                }
                                
                            }
                            ConfirmationType::Candles(candles) =>  {
                                if let Some(swing) = self.candle_count(candles){
                                    self.memory.detected_swings.push(swing)
                                }
                            }
                        }; 
        
    } 

}

