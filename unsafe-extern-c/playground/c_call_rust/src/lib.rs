mod story;
mod lint;
mod not_unsafe;

#[no_mangle]
extern "C" fn hello_world() {
    println!("Hello, world!");
}
