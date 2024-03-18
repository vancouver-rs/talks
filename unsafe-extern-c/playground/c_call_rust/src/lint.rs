#![deny(unsafe_op_in_unsafe_fn)]

unsafe fn _deref_ptr(ptr: *const u32) {
    let _a = unsafe { *ptr };
}
