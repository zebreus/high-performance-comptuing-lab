#!/usr/bin/env bash

TARGET_DIR=$1

cargo rustc --release -- -C opt-level=3 -C target-feature=+avx2,+avx,+sse2,-avx512vl,-avx512f,-avx512bw,-avx512cd,-avx512dq,-avx512vnni --cfg width_10000
cp ../target/release/lgca "${TARGET_DIR}/lgca-10000-avx2"

cargo rustc --release -- -C opt-level=3 -C target-feature=+avx2,+avx,+sse2,+avx512vl,+avx512f,+avx512bw,+avx512cd,+avx512dq,+avx512vnni --cfg width_10000 --cfg use_real_collisions_in_core
cp ../target/release/lgca "${TARGET_DIR}/lgca-10000-real"

cargo rustc --release -- -C opt-level=3 -C target-feature=+avx2,+avx,+sse2,+avx512vl,+avx512f,+avx512bw,+avx512cd,+avx512dq,+avx512vnni --cfg width_100
cp ../target/release/lgca "${TARGET_DIR}/lgca-100"

cargo rustc --release -- -C opt-level=3 -C target-feature=+avx2,+avx,+sse2,+avx512vl,+avx512f,+avx512bw,+avx512cd,+avx512dq,+avx512vnni --cfg width_1000
cp ../target/release/lgca "${TARGET_DIR}/lgca-1000"

cargo rustc --release -- -C opt-level=3 -C target-feature=+avx2,+avx,+sse2,+avx512vl,+avx512f,+avx512bw,+avx512cd,+avx512dq,+avx512vnni --cfg width_10000
cp ../target/release/lgca "${TARGET_DIR}/lgca-10000"

cargo rustc --release -- -C opt-level=3 -C target-feature=+avx2,+avx,+sse2,+avx512vl,+avx512f,+avx512bw,+avx512cd,+avx512dq,+avx512vnni --cfg width_100000
cp ../target/release/lgca "${TARGET_DIR}/lgca-100000"
