use std::mem;

pub struct Arena {
    data: Vec<u8>,
}

impl Arena {
    pub fn new() -> Self {
        Arena { data: Vec::new() }
    }

    pub fn alloc<T>(&mut self, value: T) -> *mut T {
        let size = mem::size_of::<T>();
        let align = mem::align_of::<T>();
        let len = self.data.len();
        let padding = (align - (len % align)) % align;
        let total = padding + size;
        self.data.resize(len + total, 0u8);
        let ptr_offset = len + padding;
        let ptr = unsafe {
            self.data.as_mut_ptr().add(ptr_offset) as *mut T
        };
        unsafe {
            ptr.write(value);
        }
        ptr
    }
}
