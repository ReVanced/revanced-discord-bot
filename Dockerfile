FROM gcr.io/distroless/cc
# Discord info
ARG DISCORD_AUTHORIZATION_TOKEN
ENV DISCORD_AUTHORIZATION_TOKEN $DISCORD_AUTHORIZATION_TOKEN
# MongoDB info
ARG MONGODB_URI
ENV MONGODB_URI $MONGODB_URI
COPY ./target/**/release/revanced-discord-bot /
CMD ["/revanced-discord-bot"]