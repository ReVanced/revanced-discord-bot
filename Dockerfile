FROM gcr.io/distroless/cc

COPY ./target/**/release/revanced-discord-bot /
CMD ["/revanced-discord-bot"]
