use crate::models::candle::Candle;
use std::io::Read; 
use csv::Reader;

pub struct CandleIngester<R: Read> {
    reader: Reader<R>, 
}

impl<R: Read> CandleIngester<R> {
        pub fn new( data_source: R) -> Self {
            Self {reader: Reader::from_reader(data_source)}
        }
    }

impl<R: Read> Iterator for CandleIngester<R>{
    type Item = Candle;

    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.deserialize::<Candle>().next() {
            None => None,
            Some(Ok(c)) => Some(c),
            Some(Err(e)) => {
                eprintln!("Skipping Bad row: {}", e);
                self.next()
            }
        }
    }
}


