#!/usr/bin/env bash

counter=1
NUM_THREADS=2
N=500
filename="${NUM_THREADS}_${N}_results.csv"
echo ",cpp11-pi, openMP-pi, mpi-pi++, mpi-pi" > $filename

while [ $counter -le 10 ]
do
	echo -ne $counter , >> $filename 
    echo "\nRunning cpp11-pi without slurm...";
	./cpp11-pi $NUM_THREADS 2>> $filename
	echo -ne "," >> $filename
	echo "\nRunning openMP-pi without slurm...";
	./openMP-pi $NUM_THREADS 2>> $filename
	echo -ne "," >> $filename
	echo "\nRunning mpi-pi++ without slurm...";
	mpiexec --mca btl tcp,self -n $NUM_THREADS ./mpi-pi++ 2>> $filename
	echo -ne "," >> $filename
	echo "\nRunning mpi-pi without slurm...";
	mpiexec --mca btl tcp,self -n $NUM_THREADS ./mpi-pi 2>> $filename
	echo "" >> $filename
    ((counter++))
done

echo ALL done