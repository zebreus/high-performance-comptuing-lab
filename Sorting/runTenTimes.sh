#!/usr/bin/env bash
#SBATCH -p main
#SBATCH --threads-per-core=1
#SBATCH --time=01:30:00          # total run time limit (HH:MM:SS)

set -x
#SBATCH --hint=nomultithread

NUM_NODES=$1
NUM_TASKS_PER_NODE=$2
ENTRIES=$3
TOTAL_RAM=$4
IN_FILE=$5
OUT_FILE=$6
RUN_ID=$7
EXECUTABLE=$8

NUM_TASKS=$(expr $NUM_NODES \* $NUM_TASKS_PER_NODE)
NUM_THREADS=4
RAM_PER_TASK=$(expr $TOTAL_RAM / $NUM_TASKS)
RAM_PER_CPU=$(expr $(expr $RAM_PER_TASK) / 4)

# filename=$(mktemp)
filename=$(mktemp kickoff.XXX --tmpdir)

ALGORITHM="glidesort"
if test "$NUM_TASKS" -gt 1; then
    ALGORITHM="mpi-distributed-radix-sort"
fi

ScratchDir=$(mktemp -d) # create a name for the directory
rm -rf ${ScratchDir}
mkdir -p ${ScratchDir} # make the directory
# /lustre/hdahpc/datasets/Sorting/sort1G.data
cp $EXECUTABLE ${ScratchDir}
# Copy executable to tmp on each node
srun --nodes $NUM_NODES --ntasks-per-node=$NUM_TASKS_PER_NODE --cpus-per-task=1 -- mkdir -p ${ScratchDir}
srun --nodes $NUM_NODES --ntasks-per-node=$NUM_TASKS_PER_NODE --cpus-per-task=1 -- cp $EXECUTABLE ${ScratchDir}

NAME=$(basename $IN_FILE)

min() {
    printf "%s\n" "$1" "$2" | sort -nr | head -n1
}

for LOCAL_RUN in $(seq 0 3); do
    # echo "Running openmp ($NUM_THREADS threads) on a ${MATRIX_SIZE}x${MATRIX_SIZE} matrix"
    echo -ne $NAME,$RUN_ID,$LOCAL_RUN,$NUM_NODES,$NUM_TASKS_PER_NODE,$NUM_TASKS,$ENTRIES,$TOTAL_RAM,$RAM_PER_TASK, >>$filename
    srun --nodes=$NUM_NODES --ntasks=$NUM_TASKS --ntasks-per-node=$NUM_TASKS_PER_NODE --cpus-per-task=$NUM_THREADS --mem-per-cpu=$(min ${RAM_PER_CPU} 1000)M --threads-per-core=1 -- $ScratchDir/rust-sorting --work-directory ${ScratchDir} --algorithm "$ALGORITHM" ${ScratchDir}/$NAME /tmp | grep -v "Start Singularity" >>$filename
    echo "" >>$filename
done

flock $OUT_FILE bash -c "cat $filename | awk NF >> $OUT_FILE"
rm $filename
rm -rf ${ScratchDir}
