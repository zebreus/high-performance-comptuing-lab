#!/usr/bin/env bash

INITIAL_DIR=$(pwd)

if test -z "$1"; then
    echo "No output file specified."
    exit 1
fi

filename=$(readlink -f $1)

if test -e "$filename"; then
    echo "Output file $filename already exists. Please remove it before continuing."
    exit 1
fi

if ! test -e "./compareRayonAndOpenmp.sh"; then
    echo "./matrix_openmp is missing or not executable"
    exit 1
fi

if ! test -x "./matrix_openmp"; then
    echo "./matrix_openmp is missing or not executable"
    exit 1
fi

if ! test -x "./matrix_openmp_transposed"; then
    echo "./matrix_openmp_transposed is missing or not executable"
    exit 1
fi

if ! test -x "./matrix_rayon"; then
    echo "./matrix_rayon is missing or not executable"
    exit 1
fi

if ! test -x "./matrix_generator"; then
    echo "./matrix_generator is missing or not executable"
    exit 1
fi

if test -z "$LUSTRE_HOME"; then
    echo "LUSTRE_HOME is not set"
    exit 1
fi

WORK_DIR=$(mktemp -d --tmpdir=$LUSTRE_HOME)
mkdir -p $WORK_DIR
echo $(date) >>$WORK_DIR/date.txt
cd $WORK_DIR

cp $INITIAL_DIR/matrix_openmp .
cp $INITIAL_DIR/matrix_openmp_transposed .
cp $INITIAL_DIR/matrix_rayon .
cp $INITIAL_DIR/matrix_generator .
cp $INITIAL_DIR/compareRayonAndOpenmp.sh .
chmod a+x ./compareRayonAndOpenmp.sh
output_filename=$WORK_DIR/results.csv

RUNS=30

echo "name,threads,matrix_size,run,duration,sum" >$output_filename

for NUM_THREADS in 2 4 8 16 32 64 128; do
    TEST_RUN=1
    while [ $TEST_RUN -le $RUNS ]; do
        JOBNAME="matrix-mult-$NUM_THREADS-$TEST_RUN"

        sbatch --partition=main --job-name=$JOBNAME --cpus-per-task=$NUM_THREADS --ntasks=1 --nodes=1 --output %j.out -- ./compareRayonAndOpenmp.sh $((NUM_THREADS / 2)) $output_filename $TEST_RUN
        # bash ./compareRayonAndOpenmp.sh $NUM_THREADS $filename

        ((TEST_RUN++))
    done
done

echo Working in $WORK_DIR
