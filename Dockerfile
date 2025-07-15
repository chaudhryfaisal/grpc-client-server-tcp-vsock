FROM rust:1 as build-env
RUN apt-get update && apt-get install -y protobuf-compiler
WORKDIR /app
COPY Cargo.* ./
RUN cargo fetch
ADD build.rs ./
ADD src ./src
ADD proto ./proto
RUN test -f proto/crypto.proto
ENV CARGO_BUILD_JOBS=5
RUN cargo build --release --bin server

FROM debian:12-slim
ENV SERVER_ADDR=vsock://0:5001
CMD ["/server"]
COPY --from=build-env /app/target/release/server /server
