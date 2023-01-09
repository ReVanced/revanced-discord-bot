FROM gcr.io/distroless/cc

COPY ./configuration.revanced.json /configuration.json
COPY ./target/**/release/revanced-discord-bot /
CMD ["/revanced-discord-bot"]
