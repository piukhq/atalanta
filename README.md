# Atalanta

### In Greek mythology, a renowned and swift-footed huntress.

---

A Rust application for generating transaction matching data for performance testing Harmonia.

Jira ticket: https://hellobink.atlassian.net/browse/MER-2081

Two main components, bins, for creating transaction data and uploading transactions to various locations.

src/bin/transactor.rs - Creates a raw transaction and pushed onto queues. Consumers pickup the transaction data and push them to relevant locations.

src/bin/distributor.rs - Collects the transaction data off the queues (consumers) for onward upload to various locations like API endpoints, SFTP, blob storage etc.

To run the project locally run the following commands within the root of the atlanta directory:

`cargo run --bin distributor`

`cargo run --bin transactor`

Example code is provided in the examples directory, which can be run as follows:

`cargo run --example hello`

`cargo run --example consumer`

Unit tests are executed as follows:

`cargo test -- --nocapture`

Note: --nocapture allows print statements to print during the test, if not provided print statements are not displayed.

Add packages using:

`cargo add <<package name>>`
