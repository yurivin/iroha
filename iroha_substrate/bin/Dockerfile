FROM archlinux

WORKDIR /builds

COPY iroha ./iroha
COPY node-template ./node-template
COPY bridge-tester ./bridge-tester
COPY config.json ./config.json

RUN set -eux;
RUN pacman -Syu --noconfirm
RUN pacman -S --noconfirm \
     clang lld \
     pkg-config rhash;
RUN ls -la /lib/x86_64-linux-gnu/; \
    chmod +x iroha; \
    chmod +x node-template; \
    chmod +x bridge-tester;