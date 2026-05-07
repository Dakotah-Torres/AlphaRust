use serde::{Deserialize, Deserializer};
use serde;

fn deserialize_bool_from_str<'de, D: Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
    let s = String::deserialize(d)?;
    Ok(s == "True" || s == "true")
}


#[derive(Deserialize, Clone, Copy)]
pub struct Candle {
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    #[serde(deserialize_with = "deserialize_bool_from_str")]
    pub is_closed: bool,
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
    fn new(open: f64, high: f64, low:f64, close: f64, timestamp: u64, is_closed: bool) -> Self {
        Self { 
            open, 
            high, 
            low, 
            close, 
            timestamp, 
            is_closed, 
        }
    }

    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    pub fn is_bearish(&self) -> bool {
        self.close < self.open
    }

    pub fn get_top_wick(&self) -> Result<Wick, String> {
        if self.is_closed  {
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
        if self.is_closed {
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