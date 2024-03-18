fn main() {
    cc::Build::new()
        .file("src/hello_world.c")
        .compile("hello_world");
}
