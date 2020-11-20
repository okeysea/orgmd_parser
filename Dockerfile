FROM rust:1.47.0-alpine
MAINTAINER okeysea

ARG LOCAL_UID
ARG LOCAL_GID
ENV USER=user

COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
RUN apk update
RUN apk add shadow
RUN apk add su-exec
RUN rm -rf /var/cache/apk/*

WORKDIR /app
COPY --chown=${LOCAL_UID}:${LOCAL_GID} . .

# RUN cargo install --path .
