FROM debian:buster

ARG VCS_REF=master
ARG BUILD_DATE=""
ARG REGISTRY_PATH=paritytech

WORKDIR /builds

# config for wasm32-unknown-unknown & clang
COPY base-config /root/.cargo/config

ENV RUSTUP_HOME=/usr/local/rustup \
	CARGO_HOME=/usr/local/cargo \
	PATH=/usr/local/cargo/bin:$PATH \
		CC=clang \
		CXX=clang

# download rustup
ADD "https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init" rustup-init

# install tools and dependencies
RUN set -eux; \
	apt -y update; \
	apt install -y --no-install-recommends \
		libssl-dev clang lld libclang-dev make cmake \
		git pkg-config curl time rhash ca-certificates; \
# set a link to clang
	update-alternatives --install /usr/bin/cc cc /usr/bin/clang 100; \
# install rustup
  	chmod +x rustup-init; \
  	./rustup-init -y --no-modify-path --default-toolchain stable; \
  	rm rustup-init; \
  	chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
# install Rust nightly, default is stable
	rustup install nightly; \
# install sccache
	cargo install sccache --features redis; \
# versions
	rustup show; \
	cargo --version; \
# cargo clean up
# removes compilation artifacts cargo install creates (>250M)
	rm -rf $CARGO_HOME/registry; \
# removes toolchain's html docs and autocompletions (>300M for each toolchain)
	rm -rf /usr/local/rustup/toolchains/*/share; \
# apt clean up
	apt autoremove -y; \
	apt clean; \
	rm -rf /var/lib/apt/lists/*
# cache handler
ENV	RUSTC_WRAPPER=sccache \
# show backtraces
  	RUST_BACKTRACE=1


# install tools and dependencies
RUN set -eux; \
	apt -y update; \
	apt install -y --no-install-recommends \
		chromium-driver; \
# install cargo tools
	cargo install cargo-audit cargo-web wasm-pack cargo-deny wasm-bindgen-cli; \
# install wasm toolchain
	rustup target add wasm32-unknown-unknown; \
	rustup target add wasm32-unknown-unknown --toolchain nightly; \
# install wasm-gc. It's useful for stripping slimming down wasm binaries (polkadot)
	cargo +nightly install wasm-gc; \
# versions
	rustup show; \
	cargo --version; \
# apt clean up
	apt autoremove -y; \
	apt clean; \
	rm -rf /var/lib/apt/lists/*; \
# cargo clean up
# removes compilation artifacts cargo install creates (>250M)
	rm -rf $CARGO_HOME/registry; \
# removes toolchain's html docs and autocompletions (>300M for each toolchain)
	rm -rf /usr/local/rustup/toolchains/*/share

#COPY substrate-node-template/CargoForDocker.toml /var/www/node-template/Cargo.toml
