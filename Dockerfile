FROM rust:alpine AS build

WORKDIR /cup

RUN apk add musl-dev

COPY src src
COPY Cargo.toml .
COPY Cargo.lock .

RUN cargo build --release

FROM scratch
COPY --from=build /cup/target/release/cup /cup
COPY --from=build /cup/target/release/cup.d /cup.d
ENTRYPOINT ["/cup"]