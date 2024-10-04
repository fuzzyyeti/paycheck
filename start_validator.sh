#!/bin/sh

solana-test-validator \
  --bpf-program 54oykPNNXxpXihbuU5H6j3MZmqCxaAdHALDvVYfzwnW4 ./target/deploy/paycheck.so \
  --bpf-program whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc ./paycheck/tests/fixtures/whirlpool_program.so \
  --account-dir ./web/localnet-accounts \
  --reset