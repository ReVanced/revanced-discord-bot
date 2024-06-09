FROM ubuntu:latest

COPY ./target/aarch64-unknown-linux-gnu/release/revanced-discord-bot /
CMD ["/revanced-discord-bot"]
