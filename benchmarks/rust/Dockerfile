FROM rust:1.85-slim
RUN apt-get update && apt-get install -y libssl-dev pkg-config git && rm -rf /var/lib/apt/lists/*
WORKDIR /app
RUN git clone --depth 1 https://github.com/JoaoHenriqueBarbosa/fiscal-rs.git .
WORKDIR /app/benchmarks/rust
RUN cargo build --release
CMD ["./target/release/fiscal-bench-runner"]
