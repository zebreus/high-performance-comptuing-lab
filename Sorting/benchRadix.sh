#!/usr/bin/env bash

NAME=$1

RUNS=5
FILE=benchRadix.csv
NODES=4
if test -z $NAME; then
    echo "You have to specify a name for this benchmarking run"
    exit 1
fi

if ! test -f "$FILE"; then
    echo "name,nodes,read,copy,processing,send,post,total" >"$FILE"
    exit 1
fi

unset CC && cd rust-sorting && cargo build --release --target=x86_64-unknown-linux-gnu || exit 1

cd ..

TEST_RUN=1
while [ $TEST_RUN -le $RUNS ]; do
    # JOBNAME="kickoff-$NUM_THREADS-$TEST_RUN"
    echo -ne "$NAME,$NODES," >>"$FILE"
    mpiexec -n $NODES ../target/x86_64-unknown-linux-gnu/release/rust-sorting --algorithm mpi-distributed-radix-sort /mnt/toast/10gb.dat /mnt/toast/ >>"$FILE"

    ((TEST_RUN++))
done

# mpiexec -n 4 ../target/x86_64-unknown-linux-gnu/release/rust-sorting --algorithm mpi-distributed-radix-sort ./1gb.dat /mnt/toast/
./valsort /mnt/toast/output.sorted
