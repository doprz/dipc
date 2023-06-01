FROM rust:buster as builder
COPY . /test
WORKDIR /test
RUN cargo install --bins --path .


FROM debian:bullseye-slim
COPY --from=builder /usr/local/cargo/bin/dipc /usr/local/bin/dipc
ENTRYPOINT [ "/usr/local/bin/dipc" ]
