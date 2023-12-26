use crate::runtime::ir::variant::Variant;

#[derive(Clone, PartialEq, Debug)]
pub struct Counter {
    index: usize,
    start: i64,
    step: i64,
    target: Variant,
}

impl Counter {
    pub fn new(start: i64, step: i64, target: Variant) -> Self {
        Counter {
            index: 0,
            start,
            step,
            target,
        }
    }
}

impl Iterator for Counter {
    type Item = Variant;

    fn next(&mut self) -> Option<Self::Item> {
        match self.target {
            Variant::Array(ref array) => {
                if self.index >= array.len() {
                    return None;
                }

                let value = array[self.index].clone();
                self.index += 1;
                Some(value)
            }
            Variant::Integer(end) => {
                if self.index as i64 >= end {
                    return None;
                }

                let value = Variant::Integer(self.start + (self.index as i64 * self.step));
                self.index += 1;
                Some(value)
            }
            _ => None
        }
    }
}
