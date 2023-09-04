FROM scratch
ARG TARGET_ARCH
COPY target/${TARGET_ARCH}-unknown-linux-musl/release/svgcard /
USER 1000
ENTRYPOINT ["/svgcard"]
