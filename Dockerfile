FROM ekidd/rust-musl-builder:latest AS build
ADD --chown=rust:rust . ./
RUN cargo build --release --features aws-ec2

FROM alpine:latest
COPY --from=build \
  /home/rust/src/target/x86_64-unknown-linux-musl/release/ruid \
  /usr/local/bin/

EXPOSE 8080

CMD /usr/local/bin/ruid 0

