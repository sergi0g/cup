FROM rust:alpine AS build
WORKDIR /

RUN apk add musl-dev

RUN USER=root cargo new cup
WORKDIR /cup
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

COPY src ./src
RUN cargo build --release

FROM scratch
COPY --from=build /cup/target/release/cup /cup
ENTRYPOINT ["/cup"]