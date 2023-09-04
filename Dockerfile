ARG TARGET_ARCH=x86_64

FROM scratch
COPY target/${TARGET_ARCH}-unknown-linux-musl/release/svgcard /
USER 1000
ENTRYPOINT ["/svgcard"]
