pub trait Pins: Send {
    fn set_reset(&mut self);
    fn clear_reset(&mut self);
}
