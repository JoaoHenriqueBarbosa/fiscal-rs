FROM oven/bun:latest
RUN apt-get update && apt-get install -y git && rm -rf /var/lib/apt/lists/*
WORKDIR /app
RUN git clone --depth 1 https://github.com/JoaoHenriqueBarbosa/FinOpenPOS.git .
WORKDIR /app/packages/fiscal
CMD ["bun", "run", "src/__tests__/benchmark.ts", "--json"]
