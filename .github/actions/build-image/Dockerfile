FROM --platform=$BUILDPLATFORM alpine AS builder

ARG TARGETARCH
ARG TARGETOS

COPY binaries/* /
RUN mv cup-$TARGETOS-$TARGETARCH cup
RUN chmod +x cup

FROM scratch
COPY --from=builder /cup /cup
EXPOSE 8000
ENTRYPOINT ["/cup"]