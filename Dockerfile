
# ---- Stage 1: Builder ----
FROM rust:slim AS builder

# ติดตั้งเครื่องมือที่จำเป็นสำหรับการคอมไพล์
RUN apt-get update && apt-get install -y build-essential libpq-dev

# สร้าง directory สำหรับแอปพลิเคชัน
WORKDIR /usr/src/app

# Copy เฉพาะไฟล์ที่จำเป็นสำหรับการ build dependencies ก่อน
COPY Cargo.toml Cargo.lock ./

# สร้าง dummy project เพื่อ build dependencies แยกต่างหาก
RUN mkdir src && echo "fn main() {}" > src/main.rs
# บรรทัดนี้จะทำงานได้แล้ว เพราะ Cargo ใน image ทันสมัยแล้ว
RUN cargo build --release
RUN rm -rf src

# Copy source code ทั้งหมดเข้ามาใน image
COPY . .

# Build แอปพลิเคชันใน release mode
RUN cargo build --release --bin api 

# ---- Stage 2: Runner ----
FROM debian:12-slim AS runner
RUN apt-get update && apt-get install -y libpq5 && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

COPY --from=builder /usr/src/app/migrations ./migrations
COPY --from=builder /usr/src/app/target/release/api .

CMD ["./api"]
