FROM rust:alpine AS build

RUN apk add musl-dev

RUN cargo new cup
WORKDIR /cup
COPY Cargo.toml Cargo.lock .
RUN cargo build --release

COPY src ./src
RUN cargo install --path .

FROM scratch
COPY --from=build /usr/local/cargo/bin/cup /cup
ENTRYPOINT ["/cup"]