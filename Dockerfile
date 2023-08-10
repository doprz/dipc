FROM rust:alpine as builder
RUN apk add --no-cache libc-dev
COPY . /test
WORKDIR /test
RUN cargo install --bins --path .

FROM alpine
COPY --from=builder /usr/local/cargo/bin/dipc /bin/
ENTRYPOINT [ "/bin/dipc" ]
