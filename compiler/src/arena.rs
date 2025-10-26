pub struct Arena {
    data: Vec<u8>,
}

impl Arena {
    pub fn new() -> Self {
        Arena { data: Vec::new() }
    }

    pub fn alloc<T>(&mut self, value: T) -> *mut T {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();
        let padding = (align - self.data.len() % align) % align;
        self.data.resize(self.data.len() + padding, 0);
        let ptr = self.data.as_mut_ptr().add(self.data.len()) as *mut T;
        unsafe { ptr.write(value); }
        self.data.resize(self.data.len() + size, 0);
        ptr
    }
}
