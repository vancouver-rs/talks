#[no_mangle]
extern "C" fn foo(ptr1: *const u32, ptr2: *const u32) {
    let a = unsafe { *ptr1 };
    let b = unsafe { *ptr2 };
    println!("{}{}", a, b);
}
