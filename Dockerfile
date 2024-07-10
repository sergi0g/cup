FROM rust:alpine AS build
WORKDIR /

RUN apk add musl-dev

WORKDIR /cup
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

COPY src ./src
RUN cargo build --release

FROM scratch
COPY --from=build /cup/target/release/cup /cup
ENTRYPOINT ["/cup"]