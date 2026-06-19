# Rust Assessment

There are three assignments, that highlight different parts of the work we have to do to onboard with the apis of new exchanges.

We want to respect your time, so please don't spend more than a few hours on this assessment.

## Assignment One

A lot of exchanges require users to sign the requests that they send, as a security measure.
These implementations are always slightly different, and so we need to carefully read their documentation.
The documentation can be found in `huobi_signature.md` or at https://www.huobi.com/en-us/opend/newApiPages/?id=419

You will need to implement the details of the following functions:
- `format_for_signature` on the struct `Params`
- `sign_hmac_sha256_base64`
- `generate_signature`

Then the following tests should all succeed:
- `test_format` checks that `format_for_signature` results in the correct parameter output, which is crucial for `generate_signature`.
- `test_signing` checks that `sign_hmac_sha256_base64` works correctly.
- `test_hash` and `test_hashes` check that `generate_signature` has the right output.

You should not modify the tests.

## Assignment Two

When building a strategy to trade on an exchange we want to locally keep track of the current price levels in the orderbook. 

In this assignment you should implement some parsing in `get_best_bids_and_asks_from_stream` and keep track of the levels in the book.
Once all the messages in the stream are processed the 5 best bids and 5 best asks should be returned.

You can have a look at `data_generator.rs` to see what messages you will receive in the stream.

You will need to implement the details of the following function:
- `get_best_bids_and_asks_from_stream`

Then the following test should succeed:
- `test_data` checks that the 5 bids and 5 asks on the top of the book are correctly stored.

You should not modify the tests, or `data_generator.rs`.

## Assignment Three

In a trading server there is often data that needs to be available across threads.

In this assignment you will implement two different SharedData structs that implement the same Trait, they have different behaviour with:
- LatestSharedData, on read get the latest written value, on write overwrite the latest written value, drop older values.
- OrderedSharedData, on read get the first written value, that has not been read yet.

The tests will show you the exact behaviour, and we will look a the benchmark results to assess your solution.

You should not modify the tests, or benchmarks.
