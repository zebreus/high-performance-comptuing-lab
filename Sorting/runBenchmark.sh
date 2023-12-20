#!/usr/bin/env bash
#SBATCH --threads-per-core=1
#SBATCH --time=01:30:00          # total run time limit (HH:MM:SS)

set -x

NUM_NODES=$1
NUM_TASKS_PER_NODE=$2
ENTRIES=$3
TOTAL_RAM=$4
IN_FILE=$5
OUT_FILE=$6
RUN_ID=$7
EXECUTABLE=$8
MODE=$9

NUM_TASKS=$(expr $NUM_NODES \* $NUM_TASKS_PER_NODE)
NUM_THREADS=4
RAM_PER_TASK=$(expr $TOTAL_RAM / $NUM_TASKS)
RAM_PER_CPU=$(expr $(expr $RAM_PER_TASK) / 4)

# filename=$(mktemp)
filename=$(mktemp kickoff.XXX --tmpdir)

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
    case "$MODE" in
    multi)
        # echo "Running openmp ($NUM_THREADS threads) on a ${MATRIX_SIZE}x${MATRIX_SIZE} matrix"
        echo -ne mpi-multi,$NAME,$RUN_ID,$LOCAL_RUN,$NUM_NODES,$NUM_TASKS_PER_NODE,$NUM_TASKS,$(expr $NUM_TASKS \* 4),$ENTRIES,$TOTAL_RAM,$RAM_PER_TASK, >>$filename
        srun --nodes=$NUM_NODES --ntasks=$NUM_TASKS --ntasks-per-node=$NUM_TASKS_PER_NODE --cpus-per-task=$NUM_THREADS --mem-per-cpu=${RAM_PER_CPU}M --threads-per-core=1 -- $ScratchDir/rust-sorting --work-directory ${ScratchDir} --algorithm mpi-distributed-radix-sort $IN_FILE $(mktemp --tmpdir=${ScratchDir} -d) | grep -v "Start Singularity" >>$filename
        echo "" >>$filename
        ;;
    single)
        # echo "Running openmp ($NUM_THREADS threads) on a ${MATRIX_SIZE}x${MATRIX_SIZE} matrix"
        echo -ne mpi-single,$NAME,$RUN_ID,$LOCAL_RUN,$NUM_NODES,$NUM_TASKS_PER_NODE,$NUM_TASKS,$NUM_TASKS,$ENTRIES,$TOTAL_RAM,$RAM_PER_TASK, >>$filename
        srun --nodes=$NUM_NODES --ntasks=$NUM_TASKS --ntasks-per-node=$NUM_TASKS_PER_NODE --cpus-per-task=1 --mem-per-cpu=${RAM_PER_TASK}M --threads-per-core=1 -- $ScratchDir/rust-sorting --work-directory ${ScratchDir} --algorithm mpi-radix-single-thread $IN_FILE $(mktemp --tmpdir=${ScratchDir} -d) | grep -v "Start Singularity" >>$filename
        echo "" >>$filename
        ;;
    dumb)
        echo -ne radix-sort,$NAME,$RUN_ID,$LOCAL_RUN,$NUM_NODES,$NUM_TASKS_PER_NODE,$NUM_TASKS,$NUM_TASKS,$ENTRIES,$TOTAL_RAM,$RAM_PER_TASK, >>$filename
        srun --nodes=1 --ntasks=1 --ntasks-per-node=1 --cpus-per-task=1 --mem-per-cpu=${RAM_PER_TASK}M --threads-per-core=1 -- $ScratchDir/rust-sorting --work-directory ${ScratchDir} --algorithm glidesort $IN_FILE $(mktemp --tmpdir=${ScratchDir} -d) | grep -v "Start Singularity" >>$filename
        echo "" >>$filename

        echo -ne sort-unstable,$NAME,$RUN_ID,$LOCAL_RUN,$NUM_NODES,$NUM_TASKS_PER_NODE,$NUM_TASKS,$NUM_TASKS,$ENTRIES,$TOTAL_RAM,$RAM_PER_TASK, >>$filename
        srun --nodes=1 --ntasks=1 --ntasks-per-node=1 --cpus-per-task=1 --mem-per-cpu=${RAM_PER_TASK}M --threads-per-core=1 -- $ScratchDir/rust-sorting --work-directory ${ScratchDir} --algorithm builtin-sort $IN_FILE $(mktemp --tmpdir=${ScratchDir} -d) | grep -v "Start Singularity" >>$filename
        echo "" >>$filename
        ;;

    esac

done

flock $OUT_FILE bash -c "cat $filename | awk NF >> $OUT_FILE"
rm $filename
rm -rf ${ScratchDir}
srun --nodes $NUM_NODES --ntasks-per-node=1 --cpus-per-task=1 -- rm -rf ${ScratchDir}
