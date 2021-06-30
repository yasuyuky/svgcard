FROM scratch
COPY target/x86_64-unknown-linux-musl/release/svgcard /
USER 1000
ENTRYPOINT ["/svgcard"]
