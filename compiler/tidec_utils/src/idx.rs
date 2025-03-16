pub trait Idx: 'static + Eq + PartialEq {
    fn new(idx: usize) -> Self;
    fn idx(&self) -> usize;
    fn incr(&mut self);
    fn incr_by(&mut self, by: usize);
}
