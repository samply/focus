# This assumes binaries are present, see COPY directive.
ARG IMGNAME=gcr.io/distroless/cc

FROM alpine AS chmodder
ARG TARGETARCH
COPY /artifacts/binaries-$TARGETARCH/focus /app/
RUN chmod +x /app/*

FROM ${IMGNAME}
COPY --from=chmodder /app/* /usr/local/bin/
ENTRYPOINT [ "/usr/local/bin/focus" ]

