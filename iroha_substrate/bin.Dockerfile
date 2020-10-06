FROM debian:buster

WORKDIR /builds

COPY substrate-iroha-bridge-node/bin/iroha ./iroha
COPY substrate-iroha-bridge-node/bin/substrate-iroha-bridge-node ./substrate-iroha-bridge-node
COPY substrate-iroha-bridge-node/bin/bridge-tester ./bridge-tester
COPY substrate-iroha-bridge-node/bin/iroha-tests ./unit/iroha-tests
COPY substrate-iroha-bridge-node/bin/config.json ./unit/config.json
COPY substrate-iroha-bridge-node/bin/test_config.json ./unit/tests/test_config.json
COPY config.json ./config.json

# install tools and dependencies
# set -eux; \
RUN \
	apt -y update; \
	apt install -y --no-install-recommends \
		libssl-dev lld clang \
		pkg-config; \
	apt autoremove -y; \
	apt clean; \
	rm -rf /var/lib/apt/lists/* \
	RUST_BACKTRACE=1
