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
    current_candle: Candle,
    current_candidate: Candle, 
    swing_mode: SwingMode, 
    seeking_mode: SwingType, 
    confirmation_mode: ConfirmationType,
    candle_counter: usize, 
    swing_lookback: usize,
    memory: SwingMemory,
}

impl StremingSwingDetector {
    pub fn new(current_candle: Candle, current_candidate: Candle, swing_mode: SwingMode, seeking_mode: SwingType, confirmation_mode: ConfirmationType, swing_lookback: usize ) -> Self {
        Self {
            current_candle: current_candle,
            current_candidate: current_candidate,
            swing_mode: swing_mode,
            seeking_mode: seeking_mode,
            confirmation_mode: confirmation_mode,
            swing_lookback: swing_lookback,
            memory: SwingMemory::new(swing_lookback)
        }
    }

    fn body_comapir(&self, candle: Candle) -> (f64, f64){
        (candle_body_high, candle_body_low) =  if self.candle.is_bullish() {(self.candle.close, self.candle.open)} else {(self.candle.open, self.candle.close)};

        (candle_body_high, candle_body_low)
    }

    fn percent_candle(&mut self, percent:f64) -> Swing {
        match (self.seeking_mode, self.swing_mode) {
            (SwingType::High, SwingMode::Body) => {
                let (can_high, can_low) = body_comapir(self.current_candidate);
                let swing_conf_trigger: f64 = con_low - abs(can_high - can_low) * (percent);
                self.swing_detected(swing_conf_trigger)
            }
            (SwingType::Low, SwingMode::Body) => {
                let (can_high, can_low) = body_comapir(self.current_candidate);
                let swing_conf_trigger: f64 = can_high + abs(can_high - can_low) * (percent);
                self.swing_detected(swing_conf_trigger)
            }
            (SwingType::High, SwingMode::Wick) => {
                let swing_conf_trigger: f64 = self.current_candidate.low - abs(self.current_candidate.high - self.current_candidate.low) * (percent);
                self.swing_detected(swing_conf_trigger)
            }
            (SwingType::Low, SwingMode::Wick) => {
                let swing_conf_trigger: f64 = self.current_candidate.high + abs(self.current_candidate.high - self.current_candidate.low) * (percent);
                self.swing_detected(swing_conf_trigger)
            }
        }
    }

    fn percent_price(&self, percent:f64) -> Swing {
        match (self.seeking_mode, self.swing_mode) {
            (SwingType::High, SwingMode::Body) => {
                let (can_high, can_low) = body_comapir(self.current_candidate);
                let swing_conf_trigger: f64 = con_low - (con_low *(1-percent));
                self.swing_detected(swing_conf_trigger)
            }
            (SwingType::Low, SwingMode::Body) => {
                let (can_high, can_low) = body_comapir(self.current_candidate);
                let swing_conf_trigger: f64 = can_high + (can_high  * (1-percent));
                self.swing_detected(swing_conf_trigger)
            }
            (SwingType::High, SwingMode::Wick) => {
                let swing_conf_trigger: f64 = self.current_candidate.low - (self.current_candidate.low * (1-percent));
                self.swing_detected(swing_conf_trigger)
            }
            (SwingType::Low, SwingMode::Wick) => {
                let swing_conf_trigger: f64 = self.current_candidate.high + (self.current_candidate.high * (1-percent));
                self.swing_detected(swing_conf_trigger)
            }
        }
    }

    fn ticks(&self, ticks: u32, tick_size: f64) -> f64 {
        match (self.seeking_mode, self.swing_mode) {
            (SwingType::High, SwingMode::Body) => {
                let (can_high, can_low) = body_comapir(self.current_candidate);
                let swing_conf_trigger: f64 = con_low - (ticks * tick_size);
                self.swing_detected(swing_conf_trigger)
            }
            (SwingType::Low, SwingMode::Body) => {
                let (can_high, can_low) = body_comapir(self.current_candidate);
                let swing_conf_trigger: f64 = con_low + (ticks * tick_size);
                self.swing_detected(swing_conf_trigger)
            }
            (SwingType::High, SwingMode::Wick) => {
                let swing_conf_trigger: f64 = self.current_candidate.low - (ticks * tick_size);
                self.swing_detected(swing_conf_trigger)
            }
            (SwingType::Low, SwingMode::Wick) => {
                let swing_conf_trigger: f64 = self.current_candidate.high + (ticks * tick_size);
                self.swing_detected(swing_conf_trigger)
            }
        }

    }

    fn candle_count(&mut self, count: usize) -> f64{
        self.candle_counter = count;
        match (self.seeking_mode, self.swing_mode) {
            (SwingType::High, SwingMode::Body) => {
                let (can_high, can_low) = body_comapir(self.current_candidate);
                if self.candle_counter != 0 && can_low > self.current_candle.close {
                    self.candle_counter -= 1;
                    return
                } else {
                    let swing_conf_trigger: f64 = self.current_candle.close; 
                    self.swing_detected(swing_conf_trigger)
                }
            }
            (SwingType::Low, SwingMode::Body) => {
                let (can_high, can_low) = body_comapir(self.current_candidate);
                if self.candle_counter != 0 && can_high > self.current_candle.close {
                    self.candle_counter -= 1;
                    return
                } else {
                    let swing_conf_trigger: f64 = self.current_candle.close; 
                    self.swing_detected(swing_conf_trigger)
                }
            }
            (SwingType::High, SwingMode::Wick) => {
                if self.candle_counter != 0 && can_low > self.current_candle.close {
                    self.candle_counter -= 1;
                    return
                } else {
                    let swing_conf_trigger: f64 = self.current_candle.close; 
                    self.swing_detected(swing_conf_trigger)
                }
            }
            (SwingType::Low, SwingMode::Wick) => {
                if self.candle_counter != 0 && self.current_candidate.low < self.current_candle.close {
                    self.candle_counter -= 1;
                    return
                } else {
                    let swing_conf_trigger: f64 = self.current_candle.close; 
                    self.swing_detected(swing_conf_trigger)
                }
            }
        }
    }

    fn swing_detected(&mut self, trigger: f64) -> Swing {
        match self.swing_mode {

            SwingType::High => {

                if self.current_candle.close < trigger {
                    self.seeking_mode = SwingType::Low; 
                    Swing(self.current_candidate, SwingType::High, self.current_candidate.timestamp)
                }
                else {
                    return
                }
            }
        
            SwingType::Low => {
                if self.current_candle.close > trigger {
                    self.seeking_mode = SwingType::High;
                    Swing(self.current_candidate, SwingType::Low, self.current_candidate.timestamp)
                }
                else {
                    return
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


