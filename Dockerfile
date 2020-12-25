FROM rust:1.47.0
MAINTAINER okeysea

ARG LOCAL_UID
ARG LOCAL_GID
ENV USER=user

COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]

RUN apt-get update
RUN apt-get install bash
RUN apt-get install gosu 

# RUN apk update
# RUN apk add shadow
# RUN apk add su-exec
# RUN apk add musl-dev
# RUN apk add openssl-dev
# RUN rm -rf /var/cache/apk/*

RUN cargo install wasm-pack

# for debug
# RUN apk add gdb

WORKDIR /app
COPY --chown=${LOCAL_UID}:${LOCAL_GID} . .

RUN chown -R ${LOCAL_UID}:${LOCAL_GID} /usr/local/cargo

# RUN cargo install --path .
