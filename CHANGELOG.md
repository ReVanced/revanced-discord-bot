# [1.3.0](https://github.com/revanced/revanced-discord-bot/compare/v1.2.0...v1.3.0) (2022-08-09)


### Features

* ignore casing when curing user names ([6c68e73](https://github.com/revanced/revanced-discord-bot/commit/6c68e73657a206a696444ae57f8c8e270c806b30))

# [1.2.0](https://github.com/revanced/revanced-discord-bot/compare/v1.1.0...v1.2.0) (2022-08-09)


### Features

* decancer user names ([28a19c4](https://github.com/revanced/revanced-discord-bot/commit/28a19c41204c6334e085bb52ca21d52b54ccd178))
* migrate to command framework ([ba7b82a](https://github.com/revanced/revanced-discord-bot/commit/ba7b82a6de3624ad59b587fa52ee784c3f2c9cf8))
* update ReVanced configuration to new framework ([b137217](https://github.com/revanced/revanced-discord-bot/commit/b1372177c9dc0600669df1367d98a0c15af1773b))

# [1.1.0](https://github.com/revanced/revanced-discord-bot/compare/v1.0.2...v1.1.0) (2022-07-16)


### Bug Fixes

* incorrect description for user age condition ([#16](https://github.com/revanced/revanced-discord-bot/issues/16)) ([999a345](https://github.com/revanced/revanced-discord-bot/commit/999a345f964df3af319fb2292bc48aeabc1ecae2))


### Features

* add `rustfmt` ([#14](https://github.com/revanced/revanced-discord-bot/issues/14)) ([422bed5](https://github.com/revanced/revanced-discord-bot/commit/422bed5364e8ca2711e6190d8bca59e04f9c90e9))
* default configuration for ReVanced ([8a8e94a](https://github.com/revanced/revanced-discord-bot/commit/8a8e94a9779635d3cb4348ab64a2239a38b1ff8e))

## [1.0.2](https://github.com/revanced/revanced-discord-bot/compare/v1.0.1...v1.0.2) (2022-07-09)


### Bug Fixes

* **ci:** build binaries for musl ([#9](https://github.com/revanced/revanced-discord-bot/issues/9)) ([aae1b07](https://github.com/revanced/revanced-discord-bot/commit/aae1b07edf507f9efbf5fa5434de5214aeee8464))

## [1.0.1](https://github.com/revanced/revanced-discord-bot/compare/v1.0.0...v1.0.1) (2022-07-09)


### Bug Fixes

* create release build to fix ci ([e7d26fd](https://github.com/revanced/revanced-discord-bot/commit/e7d26fdc6813c638061248c2b442b801e817b8f3))
* fix release arg for cargo build ([eab7b1d](https://github.com/revanced/revanced-discord-bot/commit/eab7b1d3ff6c1598f38502010603214b0c4b0c80))

# 1.0.0 (2022-07-09)


### Bug Fixes

* **clippy:** fix clippy warning ([96ce886](https://github.com/revanced/revanced-discord-bot/commit/96ce88666b3908d9461ab04597e2937fc6be7c90))
* clone submodules recursively ([63c351d](https://github.com/revanced/revanced-discord-bot/commit/63c351daf44c421e0e92d01d63b583444dcba38b))
* do not create unnecessary instances of `Regex` ([a3e6d88](https://github.com/revanced/revanced-discord-bot/commit/a3e6d88cfb0a7727fd5cbde9ff59c27d47ef8bbf))
* missing opening bracket ([018c8f6](https://github.com/revanced/revanced-discord-bot/commit/018c8f6e4587b5211de1f9438d29430d4e97c8d5))
* only log debug level in debug builds ([6d8eb9a](https://github.com/revanced/revanced-discord-bot/commit/6d8eb9a4044b8bf234716423467f007cae7c10ec))
* release builds not logging ([d69ae96](https://github.com/revanced/revanced-discord-bot/commit/d69ae966c5643eb41408420f372c162e48ecbcf6))
* specify github token to semantic-release ([61167ab](https://github.com/revanced/revanced-discord-bot/commit/61167abbd7b1bb48ce4df9128f36c40c0cbf3611))
* thread created should be debug, not info ([8afda24](https://github.com/revanced/revanced-discord-bot/commit/8afda248beb140678d14a4eca6ff1c3562ad315e))
* use snake case and module for configuration ([179ad3e](https://github.com/revanced/revanced-discord-bot/commit/179ad3e8244ec09ee00f922b88bb530e1550c805))
* use tracing to fix serenity logging ([704d05f](https://github.com/revanced/revanced-discord-bot/commit/704d05f45bc6cc325737f7f49655846301a10b59))


### Features

* `message-responders` & `reload` command ([8fb0ab8](https://github.com/revanced/revanced-discord-bot/commit/8fb0ab88355b31083fe14d729b411d4bb8f98118))
* `thread_introductions` ([b34c9b3](https://github.com/revanced/revanced-discord-bot/commit/b34c9b3a66a803e4c2fb129d77af24421bb97c71))
* add `rust.yml` workflow ([507656b](https://github.com/revanced/revanced-discord-bot/commit/507656b97c373f4473df922236a850a9b51bca7f))
* embeds ([4293dd5](https://github.com/revanced/revanced-discord-bot/commit/4293dd5518a95b1e5bd4fae7fd4bc20fc4c6479d))
* initial commit ([c07ad28](https://github.com/revanced/revanced-discord-bot/commit/c07ad28a60b20661d9014d4c850f906d4adbebf8))
* license ([b51c50b](https://github.com/revanced/revanced-discord-bot/commit/b51c50b7744e49e747f31280784f24c8685f656a))
* move `DISCORD_AUTHORIZATION_TOKEN` to `.env` ([42098b9](https://github.com/revanced/revanced-discord-bot/commit/42098b9db54900cae49e6adaac39cbd1630a8de1))
* update logger and other small changes ([cee17a1](https://github.com/revanced/revanced-discord-bot/commit/cee17a11eb4e4174a26e38498c385ee4b3a7c1e2))
* use cargo git dependency instead of submodule ([3c14cbe](https://github.com/revanced/revanced-discord-bot/commit/3c14cbe6a4efbb5040fe140beb3916bf8fd2e659))
