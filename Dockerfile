# Name the first stage so it can be reused later
FROM rust:latest AS builder
WORKDIR /app/revanced-discord-bot
COPY . .
RUN apt-get update && apt-get upgrade -y
RUN cargo build --release

FROM gcr.io/distroless/cc
# Discord info
ARG DISCORD_AUTHORIZATION_TOKEN
ENV DISCORD_AUTHORIZATION_TOKEN $DISCORD_AUTHORIZATION_TOKEN
# MongoDB info
ARG MONGODB_URI
ENV MONGODB_URI $MONGODB_URI
COPY --from=builder /app/revanced-discord-bot/target/release/revanced-discord-bot /
CMD ["/revanced-discord-bot"]
