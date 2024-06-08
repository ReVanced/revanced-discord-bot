FROM ubuntu:latest

COPY ./target/release/revanced-discord-bot /
CMD ["/revanced-discord-bot"]
