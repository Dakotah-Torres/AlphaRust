

pub struct Candle{
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    timestamp: u64,
    is_closed: bool
}

pub struct Wick {
    high_price: f64,
    low_price: f64
}

impl Wick {
    fn new(high_price: f64, low_price: f64) -> Self {
        Self{high_price, low_price}
    }
}

impl Candle {
    fn new(open: f64, high: f64, low:f64, close: f64, timestamp: u64, is_closed: bool) -> Self {
        Self { open, high, low, close, timestamp, is_closed }
    }

    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    pub fn is_bearish(&self) -> bool {
        self.close < self.open
    }

    pub fn top_wick(&self) -> Wick {
        if self.is_bullish() & self.is_closed {
            Wick.new(self.high, self.open);
        }
    }
}