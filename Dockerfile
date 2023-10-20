# This Dockerfile is infused with magic to speedup the build.
# In particular, it requires built binaries to be present (see COPY directive).
#
# tl;dr: To make this build work, run
#   ./dev/focusdev build
# and find your freshly built images tagged with the `localbuild` tag.

FROM alpine AS chmodder
ARG TARGETARCH
COPY /artifacts/binaries-$TARGETARCH/focus /app/
RUN chmod +x /app/*

FROM gcr.io/distroless/cc-debian12
COPY --from=chmodder /app/* /usr/local/bin/
ENTRYPOINT [ "/usr/local/bin/focus" ]

