# This assumes binaries are present, see COPY directive.

FROM alpine AS chmodder
ARG TARGETARCH
COPY /artifacts/binaries-$TARGETARCH/focus /app/
RUN chmod +x /app/*

FROM alpine
COPY --from=chmodder /app/* /usr/local/bin/
ENTRYPOINT [ "/usr/local/bin/focus" ]

