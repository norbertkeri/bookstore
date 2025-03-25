FROM rust:1.85-slim

COPY . /bookstore
WORKDIR /bookstore
RUN ["/usr/local/cargo/bin/cargo", "build"] 
CMD ["/bookstore/target/debug/bookstore"]
