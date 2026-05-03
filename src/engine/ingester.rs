use crate::detectors::candles::candle::Candle;
use crate::engine::traits::CandleSource;
use std::io::Read; 
use csv::Reader;

pub struct CsvIngester<R: Read> {
    candle_iter: csv::DeserializeRecordsIntoIter<R, Candle>,
}

pub struct ParquetIngester {}

pub struct WebhookIngester {}


impl<R: Read> CsvIngester<R> {
        pub fn new( data_source: R) -> Self {
            Self { 
                candle_iter: Reader::from_reader(data_source).into_deserialize() 
            }
        }
    }

impl<R: Read> Iterator for CsvIngester<R>{
    type Item = Candle;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.candle_iter.next() {
                Some(Ok(c)) => {
                    return Some(c);
                    
                    },
                Some(Err(e)) => {
                    eprintln!("Skipping Bad row: {}", e);
                    continue;
                }
                None => break None
            }
        }
        
    }
}

impl<R: Read> CandleSource for CsvIngester<R> {
    fn next_candle(&mut self) -> Option<Candle> {
        self.next()
    }
}
