FROM ubuntu:24.04

RUN apt-get update && apt-get install -y \
    valgrind \
    gcc \
    curl \
    vim

# Install Rust and Cargo
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install cbindgen
RUN cargo install --force cbindgen

# Install bindgen
RUN cargo install bindgen-cli
