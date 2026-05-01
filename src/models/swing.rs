use candle::Candle;

pub enum SwingType{
    High,
    Low,
}

pub struct Swing<'a>{
    pub candle:&'a Candle,
    pub swing_type: SwingType,
    pub timestamp: u64,
}

pub struct SwingBuffer<'a> {
    pub max_size: usize,
    pub buffer:  Vec<&'a Candle>,
    

}

impl<'a> SwingBuffer<'a>{
    fn new(max_size: usize) -> Self {
        Self {
            max_size,
            buffer: Vec::with_capacity(max_size)
        
        }
    }

    fn slider(&mut self, current_candle: &'a Candle){
        if self.buffer.len() == self.buffer.capacity(){
            self.buffer.remove(0);
            self.buffer.push(current_candle);
        } else {
            self.buffer.push(current_candle);
        }
    }

}

pub struct SwingDetector<'a>{
    pub max_global_size: usize,
    pub max_swing_memory: usize,
    pub rolling_window: i64,
    pub global_candle_memory: Vec<&'a Candle>,
    pub swing_buffer: SwingBuffer<'a>, 
    pub detected_swings: Option<Vec<Swing<'a>>>,
    pub current_candle: &'a Candle,
    pub global_high: Option<&'a Candle>,
    pub global_low: Option<&'a Candle>,
}

impl<'a> SwingDetector<'a> {
    fn new(max_global_size: usize, max_swing_memory: usize, rolling_window: i64, detected_swings:Vec<&'a Swing>, current_candle: &'a Candle) -> Self {
        Self {
            max_global_size,
            max_swing_memory, 
            rolling_window,
            detected_swings,
            swing_buffer: SwingBuffer::new(rolling_window),
            global_candle_memory: Vec::with_capacity(max_global_size),
            current_candle,
        }
        

    }

    fn detect_swings(&mut self){
        
        if self.swing_buffer.len() == self.swing_buffer.capacity(){
            let max = self.swing_buffer::buffer::max();
        }
        else {
            rolling_buffer::slider()
        }

    }

    
}









