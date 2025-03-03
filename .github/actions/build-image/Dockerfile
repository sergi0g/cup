FROM --platform=$BUILDPLATFORM alpine AS builder

ARG TARGETARCH
ARG TARGETOS

COPY binaries/* /
RUN mv cup-$TARGETOS-$TARGETARCH cup

FROM scratch
COPY --from=builder /cup /cup
ENTRYPOINT ["/cup"]