// Ring buffer utilities for high-rate samples.
// Invariants: fixed-capacity storage with minimal allocations on hot paths.

#[derive(Debug)]
pub struct RingBuffer<T> {
    buf: Vec<T>,
    cap: usize,
    head: usize,
    len: usize,
}

impl<T: Clone> RingBuffer<T> {
    pub fn new(cap: usize) -> Self {
        Self {
            buf: Vec::with_capacity(cap),
            cap,
            head: 0,
            len: 0,
        }
    }

    pub fn push(&mut self, item: T) {
        if self.len < self.cap {
            self.buf.push(item);
            self.len += 1;
        } else {
            self.buf[self.head] = item;
            self.head = (self.head + 1) % self.cap;
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn clear(&mut self) {
        self.buf.clear();
        self.head = 0;
        self.len = 0;
    }

    pub fn to_vec_ordered(&self) -> Vec<T> {
        let mut out = Vec::with_capacity(self.len);
        if self.len == 0 {
            return out;
        }

        if self.len < self.cap {
            out.extend(self.buf.iter().cloned());
            return out;
        }

        out.extend(self.buf[self.head..].iter().cloned());
        out.extend(self.buf[..self.head].iter().cloned());
        out
    }
}
