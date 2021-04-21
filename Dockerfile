FROM rust:1.47.0
MAINTAINER okeysea

ARG LOCAL_UID
ARG LOCAL_GID
ENV USER=user

COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]

RUN apt-get update
RUN apt-get install -y curl
RUN apt-get install bash
RUN apt-get install gosu 

# Install yarn (for package publish)
RUN apt-get update && apt-get install -y nodejs --no-install-recommends \
      && rm -rf /var/lib/apt/lists/*
RUN curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | apt-key add - \
      && echo "deb https://dl.yarnpkg.com/debian/ stable main" | tee /etc/apt/sources.list.d/yarn.list
RUN apt-get update -qq && apt-get install yarn

# RUN apk update
# RUN apk add shadow
# RUN apk add su-exec
# RUN apk add musl-dev
# RUN apk add openssl-dev
# RUN rm -rf /var/cache/apk/*

RUN cargo install wasm-pack
RUN cargo install cargo-make

# for debug
# RUN apk add gdb

WORKDIR /app
COPY --chown=${LOCAL_UID}:${LOCAL_GID} . .

RUN chown -R ${LOCAL_UID}:${LOCAL_GID} /usr/local/cargo

RUN cargo build
