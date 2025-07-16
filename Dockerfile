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
RUN cargo build --release --bin grpc_server --bin cpu_monitor

FROM debian:12-slim
ENV SERVER_ADDR=vsock://0:5001
CMD ["sh","-c","(/cpu_monitor&) && /grpc_server"]
COPY --from=build-env /app/target/release/grpc_server /grpc_server
ENV CPU_MONITOR_INTERVAL=1
ENV CPU_MONITOR_SUMMARY=3
COPY --from=build-env /app/target/release/cpu_monitor /cpu_monitor
