extern "C" fn _am_i_unsafe() {
    println!("I'm not unsafe am I?");
}

fn _regular_fn() {
    // I can call am_i_unsafe without using an unsafe block
    _am_i_unsafe();
}
