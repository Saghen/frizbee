use crate::one_shot::Appendable;

#[derive(Debug)]
pub struct ThreadSlice<T> {
    pub slice: *mut T,
    pub pos: usize,
}

unsafe impl<T: Send> Send for ThreadSlice<T> {}
unsafe impl<T: Sync> Sync for ThreadSlice<T> {}

impl<T> ThreadSlice<T> {
    pub fn new(slice: &mut [T]) -> Self {
        ThreadSlice {
            slice: slice.as_mut_ptr(),
            pos: 0,
        }
    }
}

impl<T> Appendable<T> for ThreadSlice<T> {
    fn append(&mut self, value: T) {
        unsafe { self.slice.add(self.pos).write(value) };
        self.pos += 1;
    }
}
