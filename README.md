# Atalanta

### In Greek mythology, a renowned and swift-footed huntress

---

A Rust application for generating transaction matching data for performance testing Harmonia.

Jira ticket: <https://hellobink.atlassian.net/browse/MER-2081>

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

For better output, you can [install cargo-nextest](https://nexte.st/book/pre-built-binaries.html)
and use `cargo nextest run`.

Note: --nocapture allows print statements to print during the test, if not provided print statements are not displayed.

Add packages using:

`cargo add <<package name>>`

MID's are selected at random from a subset of relevant retailer MID's which are extracted from the perf_mids.csv file in the files directory.
To create the perf_mids.csv use the following psql query, don't forget to port forward to the postgres database:

```console
psql $(kubectl get secret azure-pgfs -o json | jq -r .data.common_harmonia | base64 --decode | sed 's/bink-uksouth-.*.postgres.database.azure.com/127.0.0.1/g') -t -A -F"," -c "select LS.slug, PP.slug, MI.identifier, MI.identifier_type, MI.location_id, MI.merchant_internal_id from merchant_identifier MI, payment_provider PP, loyalty_scheme LS where MI.payment_provider_id = PP.id AND MI.loyalty_scheme_id = LS.id ORDER BY LS.slug;" > perf_mids.csv
```

## SSH/SFTP (Important!)

In order to send files over SFTP, the correct key *must* be added to the SSH agent. This can be done manually with `ssh-add`:

```sh
$ ssh-add ~/.ssh/id_sftp_example_rsa
Identity added: ~/.ssh/id_sftp_dev (user@example)
```

Check added keys with `ssh-add -l`

```sh
$ ssh-add -l
4096 SHA256:6atU6QqYFo/yM3z7fdALL2tVzMePJ/3bNhNEx9vw94g user@example (RSA)
```
