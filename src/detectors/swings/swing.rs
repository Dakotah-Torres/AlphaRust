// use crate::detectors::candles::candle::Candle;

// pub enum SwingType{
//     High,
//     Low,
// }
// pub enum SwingMode {
//     Body, 
//     Wick,
// }
// pub enum Classification {
//     Major, 
//     Intermediate, 
//     Minor,
// }

// pub struct Swing{
//     pub candle: Candle,
//     pub swing_type: SwingType,
//     pub timestamp: u64,
//     pub dominance: usize,
//     pub swing_classification: Classification,
// }

// impl Swing {
//     fn new(candle: Candle, swing_type: SwingType, timestamp: u64 ) -> Self {
//         Self{
//             candle: candle,
//             swing_type: swing_type,
//             timestamp: timestamp,
//             dominance: 1, 
//             swing_classification: Classification::Minor, 

//         }
//     }
// }
// pub struct SwingBuffer {
//     pub max_size: usize,
//     pub buffer:  Vec<Candle>,
    

// }

// impl SwingBuffer{
//     pub fn new(max_size: usize) -> Self {
//         Self {
//             max_size,
//             buffer: Vec::with_capacity(max_size)
        
//         }
//     }

//     pub fn slider(&mut self, current_candle: Candle){
//         if self.buffer.len() == self.buffer.capacity(){
//             self.buffer.remove(0);
//             self.buffer.push(current_candle);
//         } else {
//             self.buffer.push(current_candle);
//         }
//     }

// }
// pub struct SwingMemory {
//     pub lookback: usize,
//     pub detected_swings: Vec<Swing>,
// }

// impl SwingMemory {
//     fn new( lookback: usize, ) -> Self {
//         Self {
//             lookback: lookback,
//             detected_swings: Vec::with_capacity(lookback),
//         }
//     }
// }
// pub struct SwingDetector{
//     pub rolling_window: usize,
//     pub global_lookback: usize,
//     pub major_threshold: f64, 
//     pub intermediate_threshold: f64,
//     pub current_candle: Candle,
//     pub swing_buffer: SwingBuffer, 
//     pub swing_memory: SwingMemory
// }

// impl SwingDetector{
//     pub fn new( rolling_window: usize, global_lookback: usize, major_threshold: f64, intermediate_threshold: f64,  current_candle: Candle) -> Self {
//         Self {
//             rolling_window: rolling_window,
//             global_lookback: global_lookback,
//             major_threshold: major_threshold,
//             intermediate_threshold: intermediate_threshold,
//             current_candle,
//             swing_buffer: SwingBuffer::new(rolling_window),
//             swing_memory:SwingMemory::new(global_lookback),
            
//         }
        

//     }

//     fn detect_swing_highs_wick(&mut self) {

//         let buf = &self.swing_buffer.buffer; 
//         let len = buf.len();

//         if len < 3 { return; }


//         let interior = &buf[1..len-1];

//         if let Some(highest) = self.swing_buffer.buffer
//             .iter()
//             .max_by(|a, b| a.high.partial_cmp(&b.high).unwrap()){
//                 if highest.high > buf[0].high && highest.high > buf[len - 1].high {
//                 let swing = Swing::new(highest.clone(), SwingType::High, highest.timestamp);
//                 self.swing_memory.detected_swings.push(swing);
//             }
//         }
//     }

//     fn detect_swing_lows_wick(&mut self) {
//         if let Some(lowest) = self.swing_buffer.buffer
//             .iter()
//             .min_by(|a,b| a.low.partial_cmp(&b.low).unwrap()){
//                 let swing = Swing::new(lowest.clone(), SwingType::Low, lowest.timestamp);
//                 self.swing_memory.detected_swings.push(swing);
//             }
//     }

//     fn detect_swing_highs_body(&mut self) {
//         if let Some(high_body) = self.swing_buffer.buffer.iter()
//             .max_by(|a,b| {
//                 let a_high = if a.is_bullish() {a.close} else {a.open};
//                 let b_high = if b.is_bullish() {b.close} else {b.open};
//                 a_high.partial_cmp(&b_high).unwrap()
//             }){
//                 let swing = Swing::new(high_body.clone(), SwingType::High, high_body.timestamp);
//                 self.swing_memory.detected_swings.push(swing);
//             }
//     }

//     fn detect_swing_lows_body(&mut self) {
//         if let Some(low_body) = self.swing_buffer.buffer
//             .iter()
//             .min_by( |a,b|
//                 {
//                     let a_low = if a.is_bearish() {a.close} else {a.open};
//                     let b_low = if b.is_bearish() {b.close} else {b.open};
//                     a_low.partial_cmp(&b_low).unwrap()
//                 }){
//                     let swing = Swing::new(low_body.clone(), SwingType::Low, low_body.timestamp);
//                     self.swing_memory.detected_swings.push(swing);
//                 }
        

//     }

//     pub fn detect_swings(&mut self, swing_mode: SwingMode, swing_type: SwingType){
//         match swing_mode {
//             SwingMode::Body => {
//                 match swing_type {
//                     SwingType::High => self.detect_swing_highs_body(),
//                     SwingType::Low => self.detect_swing_lows_body(),
//                 }
//             }

//             SwingMode::Wick => {
//                 match swing_type {
//                     SwingType::High => self.detect_swing_highs_wick(),
//                     SwingType::Low => self.detect_swing_lows_wick(),
//                 }
//             }
//         }
            
        
//     }
    

// }








