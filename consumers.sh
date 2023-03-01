#!/bin/sh
PAYMENT_SLUG=visa PAYMENT_TYPE=auth cargo run --bin distributor &
PAYMENT_SLUG=visa PAYMENT_TYPE=settlement cargo run --bin distributor &
RETAILER_SLUG=wasabi-club cargo run --bin distributor