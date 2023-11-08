#!/usr/bin/env bash
#SBATCH --hint=multithread
NUM_THREADS=$1
OUT_FILE=$2

# filename=$(mktemp)
filename=$(mktemp kickoff.XXX --tmpdir)

for N in 1 2 4 8 16 32 64 128 256 512 1024 2048 4096 8192 16384; do
    echo "Running cpp11-pi without slurm..."
    echo -ne cpp11-pi,$NUM_THREADS,$N, >>$filename
    ./cpp11-pi $NUM_THREADS $N 2>>$filename
    echo "" >>$filename

    echo "Running openMP-pi without slurm..."
    echo -ne openMP-pi,$NUM_THREADS,$N, >>$filename
    ./openMP-pi $NUM_THREADS $N 2>>$filename
    echo "" >>$filename

    echo "Running mpi-pi without slurm..."
    echo -ne mpi-pi,$NUM_THREADS,$N, >>$filename
    mpirun --oversubscribe -n $NUM_THREADS -- ./mpi-pi $N 2>>$filename
    echo "" >>$filename

    echo "Running mpi-pi++ without slurm..."
    echo -ne mpi-pi++,$NUM_THREADS,$N, >>$filename
    mpirun --oversubscribe -n $NUM_THREADS -- ./mpi-pi++ $N 2>>$filename
    echo "" >>$filename
done

flock $OUT_FILE bash -c "cat $filename | awk NF >> $OUT_FILE"
rm $filename
