FROM debian:experimental

WORKDIR /builds

COPY iroha ./iroha
COPY node-template ./node-template
COPY config.json ./config.json

RUN set -eux; \
	apt -y update; \
	apt install -y --no-install-recommends \
     clang lld libclang-dev \
     pkg-config rhash ; \
    ls -la /lib/x86_64-linux-gnu/; \
    chmod +x iroha; \
    chmod +x node-template;