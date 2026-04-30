use candel::Candle;

pub enum SwingType{
    swing_high,
    swing_low,
}

pub struct Swing{
    candle: Candle,
    timestamp: u64,
    swing_type: SwingType
}

impl Swing {
    fn new(candle: Candle, timestamp:u64, swing_type:SwingType) -> Self {
        Self {candle, timestamp, swing_type}
    }
}