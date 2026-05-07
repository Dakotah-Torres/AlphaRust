use crate::detectors::primatives::candles::candle::Candle;


pub trait CandleSource {
    fn next_candle(&mut self) -> Option<Candle>;
}