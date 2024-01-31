#!/usr/bin/env bash
filename="results.csv"

RUNS=20

echo "name,threads,n,duration" >$filename

for NUM_THREADS in 1 2 4 8 16 32 64 128; do
    TEST_RUN=1
    while [ $TEST_RUN -le $RUNS ]; do
        JOBNAME="kickoff-$NUM_THREADS-$TEST_RUN"

        sbatch --partition=main --job-name=$JOBNAME --cpus-per-task=$NUM_THREADS --ntasks=1 --nodes=1 --output %j.out -- ./batchJobs.sh $NUM_THREADS $filename

        ((TEST_RUN++))
    done
done
