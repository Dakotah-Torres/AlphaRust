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
            swing_mode: swing_mode,
            seeking_mode: seeking_mode,
            confirmation_mode: confirmation_mode,
            swing_lookback: swing_lookback,
            memory: SwingMemory::new(swing_lookback)
        }
    }
    fn clean_candles(&mut self) {
        self.current_candidate = None; 
        self.current_candle = None;
    }  

    fn process_candle(&mut self, incoming_candle: Candle) {
        if let Some(current_candle) = self.current_candle {
            if let Some(cur_candidate) = self.current_candidate {
                (can_high, can_low) = get_candidate_levels(&cur_candidate); 
                (high, low) = get_candidate_levels(current_candle);
                match self.seeking_mode {
                    SwingType::High => {
                        if can_high < current_candle.close {
                            self.current_candidate = Some(self.current_candle);
                            self.current_candle = incoming_candle;
                        }

                    }
                    SwingType::Low => {
                        if can_low > current_candidate.close {
                            self.current_candidate = Some(self.current_candle);
                            self.current_candle = incoming_candle;
                        }
                    }
                }

            } else {
                self.current_candidate = curr_candle;
                return
            }
        } else {
            self.current_candle = incoming_candle; 
            return
        }
        
    }

    fn get_candidate_levels(&self, candle: &Candle){
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
                SwingType::Low => high +  (high - low) * candle, 
            }; 

            self.swing_detected(trigger)
        } 
        else {
            None
        }
    }

    fn percent_price(&self, percent:f64) -> Option<Swing> {
        if let Some(candidate) = &self.current_candidate {
            let (high, low) = self.get_candidate_levels(candidate);
            let trigger = match self.seeking_mode {
                SwingType::High => low - (low * (1-percent)),
                SwingType::Low => high + (high * (1 - percent)), 
            }; 
            self.swing_detected(trigger)
        } else {
            None
        }
    }

    fn ticks(&self, ticks: u64, tick_size: f64) -> Option<Swing> {
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

    fn swing_confirmation(&self){
        match self.confirmation_mode {
                            ConfirmationType::PercentCandle(percent) => {
                                self.SwingMemory.push(self.percent_candle(percent))
                            },
                            ConfirmationType::PercentPrice(percent) => {
                                self.SwingMemory.push(self.percent_price(percent))
                            }
                            ConfirmationType::Ticks(ticks, tick_size) => {  
                                self.SwingMemory.push(self.ticks(ticks, tick_size))
                            }
                            ConfirmationType::Candles(candles) =>  {
                                self.SwingMemory.push(self.candle_count(candles))
                
                            }
                        }; 
        
    } 

}

