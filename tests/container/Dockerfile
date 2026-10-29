# rsh server with interactive CLI tools, for end-to-end checks from the host:
#   docker build -f tests/container/Dockerfile -t rsh-e2e .
#   docker run -d --name rsh-e2e -p 127.0.0.1:47322:2222/udp rsh-e2e
# The token below is a fixed test value; publish the port on loopback only.
#   echo "rsh-e2e e2e-token" > auth; chmod 600 auth; RSH_AUTH=auth RSH_PEER=127.0.0.1:47322 rsh rsh-e2e
FROM rust:1-slim-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock build.rs ./
COPY src src
RUN cargo build --release --locked

FROM node:22-bookworm-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends tmux vim-tiny less procps ncurses-term git ca-certificates \
 && rm -rf /var/lib/apt/lists/*
RUN npm install -g @anthropic-ai/claude-code @openai/codex && npm cache clean --force
COPY --from=build /src/target/release/rsh /usr/local/bin/rsh
RUN useradd -m -s /bin/bash dev \
 && mkdir -p /home/dev/.rsh \
 && echo "client:e2e e2e-token" > /home/dev/.rsh/authorized \
 && chown -R dev:dev /home/dev/.rsh && chmod 700 /home/dev/.rsh && chmod 600 /home/dev/.rsh/authorized
USER dev
WORKDIR /home/dev
ENV LANG=C.UTF-8
EXPOSE 2222/udp
CMD ["rsh", "serve", "--bind", "0.0.0.0:2222", "--authorized", "/home/dev/.rsh/authorized"]
