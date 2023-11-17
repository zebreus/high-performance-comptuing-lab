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

    echo "Running openmp ($NUM_THREADS threads) on a ${MATRIX_SIZE}x${MATRIX_SIZE} matrix"
    echo -ne openmp,$NUM_THREADS,$MATRIX_SIZE,$RUN_ID, >>$filename
    ./matrix_openmp $MATRIX_A_FILE $MATRIX_B_FILE --threads $NUM_THREADS >>$filename
    echo "" >>$filename

    echo "Running openmp-inverted ($NUM_THREADS threads) on a ${MATRIX_SIZE}x${MATRIX_SIZE} matrix"
    echo -ne openmp-inverted,$NUM_THREADS,$MATRIX_SIZE,$RUN_ID, >>$filename
    ./matrix_openmp_inverted $MATRIX_A_FILE $MATRIX_B_FILE --threads $NUM_THREADS >>$filename
    echo "" >>$filename

    echo "Running rayon ($NUM_THREADS threads) on a ${MATRIX_SIZE}x${MATRIX_SIZE} matrix"
    echo -ne rayon,$NUM_THREADS,$MATRIX_SIZE,$RUN_ID, >>$filename
    ./matrix_rayon $MATRIX_A_FILE $MATRIX_B_FILE --threads $NUM_THREADS --algorithm no-indices >>$filename
    echo "" >>$filename

    echo "Running rayon-faithful-pairs ($NUM_THREADS threads) on a ${MATRIX_SIZE}x${MATRIX_SIZE} matrix"
    echo -ne rayon-faithful-pairs,$NUM_THREADS,$MATRIX_SIZE,$RUN_ID, >>$filename
    ./matrix_rayon $MATRIX_A_FILE $MATRIX_B_FILE --threads $NUM_THREADS --algorithm faithful-pairs >>$filename
    echo "" >>$filename

    echo "Running rayon-faithful-mutex ($NUM_THREADS threads) on a ${MATRIX_SIZE}x${MATRIX_SIZE} matrix"
    echo -ne rayon-faithful-mutex,$NUM_THREADS,$MATRIX_SIZE,$RUN_ID, >>$filename
    ./matrix_rayon $MATRIX_A_FILE $MATRIX_B_FILE --threads $NUM_THREADS --algorithm faithful-mutex >>$filename
    echo "" >>$filename

    echo "Running rayon-faithful-iterators ($NUM_THREADS threads) on a ${MATRIX_SIZE}x${MATRIX_SIZE} matrix"
    echo -ne rayon-faithful-iterators,$NUM_THREADS,$MATRIX_SIZE,$RUN_ID, >>$filename
    ./matrix_rayon $MATRIX_A_FILE $MATRIX_B_FILE --threads $NUM_THREADS --algorithm faithful-iterators >>$filename
    echo "" >>$filename

    echo "Running rayon-faithful-unsafe ($NUM_THREADS threads) on a ${MATRIX_SIZE}x${MATRIX_SIZE} matrix"
    echo -ne rayon-faithful-unsafe,$NUM_THREADS,$MATRIX_SIZE,$RUN_ID, >>$filename
    ./matrix_rayon $MATRIX_A_FILE $MATRIX_B_FILE --threads $NUM_THREADS --algorithm faithful-unsafe >>$filename
    echo "" >>$filename

    rm $MATRIX_A_FILE $MATRIX_B_FILE
done

flock $OUT_FILE bash -c "cat $filename | awk NF >> $OUT_FILE"
rm $filename
