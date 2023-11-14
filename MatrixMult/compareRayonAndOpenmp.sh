#!/usr/bin/env bash
#SBATCH --hint=multithread
NUM_THREADS=$1
OUT_FILE=$2
RUN_ID=$3

# filename=$(mktemp)
filename=$(mktemp kickoff.XXX --tmpdir)

for MATRIX_SIZE in 1 2 4 8 16 32 64 128 256 512 1024 2048 4096; do
    MATRIX_A_FILE=$(mktemp matrix_${MATRIX_SIZE}x${MATRIX_SIZE}.XXX --tmpdir)
    MATRIX_B_FILE=$(mktemp matrix_${MATRIX_SIZE}x${MATRIX_SIZE}.XXX --tmpdir)

    echo "Generating ${MATRIX_SIZE}x${MATRIX_SIZE} matrices"
    ./matrix_generator $MATRIX_SIZE $MATRIX_SIZE >$MATRIX_A_FILE
    ./matrix_generator $MATRIX_SIZE $MATRIX_SIZE >$MATRIX_B_FILE

    echo "Running with openMP ($NUM_THREADS threads) on a ${MATRIX_SIZE}x${MATRIX_SIZE} matrix"
    echo -ne openmp,$NUM_THREADS,$MATRIX_SIZE,$RUN_ID, >>$filename
    ./matrix_openmp $MATRIX_A_FILE $MATRIX_B_FILE --threads $NUM_THREADS >>$filename
    echo "" >>$filename

    echo "Running with rayon ($NUM_THREADS threads) on a ${MATRIX_SIZE}x${MATRIX_SIZE} matrix"
    echo -ne rayon,$NUM_THREADS,$MATRIX_SIZE,$RUN_ID, >>$filename
    ./matrix_rayon $MATRIX_A_FILE $MATRIX_B_FILE --threads $NUM_THREADS >>$filename
    echo "" >>$filename

    rm $MATRIX_A_FILE $MATRIX_B_FILE
done

flock $OUT_FILE bash -c "cat $filename | awk NF >> $OUT_FILE"
rm $filename
