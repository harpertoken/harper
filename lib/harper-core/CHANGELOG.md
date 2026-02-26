# Changelog

## [0.6.0](https://github.com/harpertoken/harper/compare/harper-core-0.5.0...harper-core-0.6.0) (2026-02-26)


### Features

* add screenpipe integration for screen/audio search ([#133](https://github.com/harpertoken/harper/issues/133)) ([89603b6](https://github.com/harpertoken/harper/commit/89603b61d61495b453be788c9942843b2723a779))


### Bug Fixes

* tool calls for openai/sambanova ([#144](https://github.com/harpertoken/harper/issues/144)) ([57105b4](https://github.com/harpertoken/harper/commit/57105b43f8ee12d4807dcc9cb752fb12bbca33b7))

## [0.5.0](https://github.com/harpertoken/harper/compare/harper-core-0.4.0...harper-core-0.5.0) (2026-02-13)


### Features

* add workspace crate for benchmarking and integration tests ([#95](https://github.com/harpertoken/harper/issues/95)) ([16da461](https://github.com/harpertoken/harper/commit/16da4618689c19cd37400c72422d7b66a0bcc78a))
* audit logs ([#124](https://github.com/harpertoken/harper/issues/124)) ([8b07ad3](https://github.com/harpertoken/harper/commit/8b07ad3525626aad2d1ddf1734201e93c6864797))


### Bug Fixes

* **gemini:** update authentication header to x-goog-api-key ([47b4c6f](https://github.com/harpertoken/harper/commit/47b4c6f47381cf74219acb863a284a09274913e3))
* prevent multiple api keys from overwriting each other ([#107](https://github.com/harpertoken/harper/issues/107)) ([49f0fbb](https://github.com/harpertoken/harper/commit/49f0fbbb373787f78b17ae44a650fb75b7407446))
* remove sensitive data from test assertions ([9c29b77](https://github.com/harpertoken/harper/commit/9c29b77dca119e1d2fcd91ec995066daa0f3c53e))
* surface sqlite errors ([#123](https://github.com/harpertoken/harper/issues/123)) ([0b3860f](https://github.com/harpertoken/harper/commit/0b3860f19286242673ad5c34e0bd9d282ddbf355))
* trigger release workflow ([#128](https://github.com/harpertoken/harper/issues/128)) ([2abc192](https://github.com/harpertoken/harper/commit/2abc192c3def58a92e9eb268b7752cdd990ffbc3))
* trigger release workflow again ([#129](https://github.com/harpertoken/harper/issues/129)) ([26114dd](https://github.com/harpertoken/harper/commit/26114ddf51c48c0eecca3be913f1b9ca51a1aabf))


### Refactors

* remove code duplication in error display ([#108](https://github.com/harpertoken/harper/issues/108)) ([7d36218](https://github.com/harpertoken/harper/commit/7d362189380f3ae6352da2a8fa5378b5136336e2))
* split harper into core and UI crates ([#90](https://github.com/harpertoken/harper/issues/90)) ([9d79ed7](https://github.com/harpertoken/harper/commit/9d79ed738b549bceca953b3191cc556d7b71d482))


### Chores

* merge remote changes ([505b853](https://github.com/harpertoken/harper/commit/505b8531eba5b2e479f8bfa5aa6ef6579afe81ba))
* release main ([#94](https://github.com/harpertoken/harper/issues/94)) ([26af4cb](https://github.com/harpertoken/harper/commit/26af4cbb9f3be0fbaf0715321a0391e529778510))


### CI

* add cargo-dist packaging and harden linting ([#121](https://github.com/harpertoken/harper/issues/121)) ([b20c214](https://github.com/harpertoken/harper/commit/b20c214c8b5075920861e5d49935d198344c2e66))

## [0.4.0](https://github.com/harpertoken/harper/compare/harper-core-v0.3.4...harper-core-v0.4.0) (2026-01-18)


### Features

* add workspace crate for benchmarking and integration tests ([#95](https://github.com/harpertoken/harper/issues/95)) ([16da461](https://github.com/harpertoken/harper/commit/16da4618689c19cd37400c72422d7b66a0bcc78a))


### Bug Fixes

* remove sensitive data from test assertions ([9c29b77](https://github.com/harpertoken/harper/commit/9c29b77dca119e1d2fcd91ec995066daa0f3c53e))


### Refactors

* split harper into core and UI crates ([#90](https://github.com/harpertoken/harper/issues/90)) ([9d79ed7](https://github.com/harpertoken/harper/commit/9d79ed738b549bceca953b3191cc556d7b71d482))


### Chores

* merge remote changes ([505b853](https://github.com/harpertoken/harper/commit/505b8531eba5b2e479f8bfa5aa6ef6579afe81ba))
