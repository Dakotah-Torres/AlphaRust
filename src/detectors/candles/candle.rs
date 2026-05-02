use serde::{Deserialize};
use serde;


#[derive(Deserialize)]
pub struct Candle{
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub is_closed: String, 
}

pub struct Wick {
    pub high_price: f64,
    pub low_price: f64,
    pub timestamp: u64
}
impl Wick {
    fn new(high_price: f64, low_price: f64, timestamp: u64) -> Self {
        Self{high_price, low_price, timestamp}
    }
}

impl Candle {
    fn new(open: f64, high: f64, low:f64, close: f64, timestamp: u64, is_closed: String) -> Self {
        Self { open, high, low, close, timestamp, is_closed }
    }

    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    pub fn is_bearish(&self) -> bool {
        self.close < self.open
    }

    pub fn get_top_wick(&self) -> Result<Wick, String> {
        if self.is_closed == "True" {
            if self.is_bullish() {
                let top_wick = Wick::new(self.high, self.close, self.timestamp);
                Ok(top_wick)
            } 
            else if self.is_bearish() {
                let top_wick = Wick::new(self.high, self.open, self.timestamp);
                Ok(top_wick)
            }
            else {
                return  Err(format!("Invalid Candle Type"));
            }
            
        } else {
            return  Err(format!("Candle Not Closed"));
        }
    }

    pub fn get_bottom_wick(&self) -> Result<Wick, String> {
        if self.is_closed == "True" {
            if self.is_bullish() {
                let bottom_wick = Wick::new(self.open, self.low, self.timestamp);
                Ok(bottom_wick)
            }
            else if self.is_bearish() {
                let bottom_wick = Wick::new(self.close, self.low, self.timestamp);
                Ok(bottom_wick)
            }
            else {
                return  Err(format!("Invalid Candle Type"));
            }
        }
        else {
            return  Err(format!("Candle Not Closed"));
        }
    }

    
}