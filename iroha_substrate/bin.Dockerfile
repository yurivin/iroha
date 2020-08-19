FROM debian:buster

WORKDIR /builds

COPY substrate-iroha-bridge-node/bin/iroha ./iroha
COPY substrate-iroha-bridge-node/bin/substrate-iroha-bridge-node ./substrate-iroha-bridge-node
COPY substrate-iroha-bridge-node/bin/bridge-tester ./bridge-tester
COPY config.json ./config.json

# install tools and dependencies
# set -eux; \
RUN \
	apt -y update; \
	apt install -y --no-install-recommends \
		libssl-dev lld clang \
		pkg-config; \
# 	apt install -y --no-install-recommends \
# 		libssl-dev clang lld libclang-dev make cmake \
# 		git pkg-config curl time rhash ca-certificates; \
# set a link to clang
# 	update-alternatives --install /usr/bin/cc cc /usr/bin/clang 100; \
	apt autoremove -y; \
	apt clean; \
	rm -rf /var/lib/apt/lists/* \
	RUST_BACKTRACE=1
