FROM rust:alpine AS build
WORKDIR /

RUN apk add musl-dev

RUN USER=root cargo new --bin cup
WORKDIR /cup

COPY Cargo.toml Cargo.lock .
RUN cargo build --release
RUN rm -rf src/

COPY src src
# This is a very bad workaround, but cargo only triggers a rebuild this way for some reason
RUN printf "\n" >> src/main.rs
RUN cargo build --release

FROM scratch
COPY --from=build /cup/target/release/cup /cup
ENTRYPOINT ["/cup"]