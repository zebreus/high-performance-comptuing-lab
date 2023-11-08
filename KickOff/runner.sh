#!/usr/bin/env bash
filename="results.csv"

RUNS=10

echo "name,threads,n,run,duration" >$filename

for N in 1 2 4 8 16 32 64 128 256 512 1024 2048 4096 8192; do
	for NUM_THREADS in 1 2 4 8; do
		counter=1
		while [ $counter -le $RUNS ]; do
			echo "Running cpp11-pi without slurm..."
			echo -ne cpp11-pi,$NUM_THREADS,$N,$counter, >>$filename
			./cpp11-pi $NUM_THREADS $N 2>>$filename
			echo "" >>$filename

			echo "Running openMP-pi without slurm..."
			echo -ne openMP-pi,$NUM_THREADS,$N,$counter, >>$filename
			./openMP-pi $NUM_THREADS $N 2>>$filename
			echo "" >>$filename

			echo "Running mpi-pi without slurm..."
			echo -ne mpi-pi,$NUM_THREADS,$N,$counter, >>$filename
			mpiexec --mca btl tcp,self -n $NUM_THREADS ./mpi-pi $N 2>>$filename
			echo "" >>$filename

			echo "Running mpi-pi++ without slurm..."
			echo -ne mpi-pi++,$NUM_THREADS,$N,$counter, >>$filename
			mpiexec --mca btl tcp,self -n $NUM_THREADS ./mpi-pi++ $N 2>>$filename
			echo "" >>$filename

			((counter++))
		done
	done
done
