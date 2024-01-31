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

if ! test -e "./runTenTimes.sh"; then
    echo "./runTenTimes.sh is missing or not executable"
    exit 1
fi

if ! test -x "../target/release/rust-sorting"; then
    echo "../target/release/rust-sorting is missing or not executable"
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

cp $INITIAL_DIR/../target/release/rust-sorting .
cp $INITIAL_DIR/runTenTimes.sh .

chmod a+x $WORK_DIR/runTenTimes.sh
output_filename=$WORK_DIR/results.csv

RUNS=30

echo "name,batch,run,nodes,tasks_per_node,tasks,entries,total_ram,ram_per_task,reading_the_input,dividing_the_input_into_buckets,sending_to_workers,writing_to_disk,receiving_from_workers,fetching_time_from_workers,receiving_on_worker,sorting_on_worker,sending_to_manager,duration" >$output_filename

get-file-with-entries() {
    case $1 in

    100)
        echo -n "/lustre/hdahpc/datasets/Sorting/sort100.data"
        ;;

    1000)
        echo -n "/lustre/hdahpc/datasets/Sorting/sort1K.data"
        ;;

    1000000)
        echo -n "/lustre/hdahpc/datasets/Sorting/sort1M.data"
        ;;

    16000000)
        echo -n "/lustre/hdahpc/datasets/Sorting/sort16M.data"
        ;;

    64000000)
        echo -n "/lustre/hdahpc/datasets/Sorting/sort64M.data"
        ;;

    256000000)
        echo -n "/lustre/hdahpc/datasets/Sorting/sort256M.data"
        ;;

    1000000000)
        echo -n "/lustre/hdahpc/datasets/Sorting/sort1G.data"
        ;;

    *)
        echo "Unsupported number of entries" >&2
        exit 1
        ;;
    esac
}

# in megabytes across all workers
# Currently double the total dataset, but min 4G, because I am too lazy to calculate the exact amount
get-ram-requirements() {
    case $1 in

    100)
        echo -n 4000
        ;;

    1000)
        echo -n 4000
        ;;

    1000000)
        echo -n 4000
        ;;

    16000000)
        echo -n 6400
        ;;

    64000000)
        echo -n 25600
        ;;

    256000000)
        echo -n 102400
        ;;

    1000000000)
        echo -n 400000
        ;;

    *)
        echo "Unsupported number of entries" >&2
        exit 1
        ;;
    esac
}

min() {
    printf "%s\n" "$1" "$2" | sort -nr | head -n1
}

run-benchmark() {
    NUM_NODES=$1
    NUM_TASKS_PER_NODE=$2
    ENTRIES=$3
    OUT_FILE=$4
    RUN_ID=$5
    EXECUTABLE=$6

    IN_FILE=$(get-file-with-entries $ENTRIES)
    TOTAL_RAM=$(get-ram-requirements $ENTRIES)

    NUM_TASKS=$(expr $NUM_NODES \* $NUM_TASKS_PER_NODE)
    RAM_PER_TASK=$(expr $TOTAL_RAM / $NUM_TASKS)
    RAM_PER_CPU=$(expr $(expr $RAM_PER_TASK) / 4)

    JOBNAME="sorting-$NUM_NODES-$NUM_TASKS_PER_NODE-${RAM_PER_TASK}G-$RUN_ID"

    sbatch --partition=main --job-name=$JOBNAME --nodes=${NUM_NODES} --ntasks-per-node=${NUM_TASKS_PER_NODE} --cpus-per-task=4 --mem-per-cpu=$(min $(expr $RAM_PER_CPU \* 2) 1000)M --threads-per-core=1 --output=${WORK_DIR}/%j.out --error=${WORK_DIR}/%j.err -- ./runTenTimes.sh $NUM_NODES $NUM_TASKS_PER_NODE $ENTRIES $TOTAL_RAM $IN_FILE $OUT_FILE $RUN_ID $EXECUTABLE
}

# run-benchmark 4 4 1000 $output_filename 1 $WORK_DIR/rust-sorting

# for ENTRIES in 100 1000 1000000 16000000 64000000 256000000 1000000000 ; do
# for ENTRIES in 100 1000 1000000; do
# for ENTRIES in 1000000 16000000; do

for ITERATION in 1 2 3 4; do
    for NODES in 2 4 8 16 32; do
        for ENTRIES in 100 1000 1000000 16000000 64000000 256000000 1000000000; do
            RUNN=$((RUN++))
            echo RUN: $RUNN
            run-benchmark $NODES 1 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting
        done
    done
done

# for NUM_THREADS in 2 4 8 16 32 64 128; do
#     TEST_RUN=1
#     while [ $TEST_RUN -le $RUNS ]; do
#         JOBNAME="sorting-$NUM_NODES-$NUM_TASKS-$TEST_RUN"

#         sbatch --partition=main --job-name=$JOBNAME --cpus-per-task=$NUM_THREADS --ntasks=1 --nodes=1 --output $WORKDIR/%j.out --error $WORKDIR/%j.err -- ./compareRayonAndOpenmp.sh $((NUM_THREADS / 2)) $output_filename $TEST_RUN
#         # bash ./compareRayonAndOpenmp.sh $NUM_THREADS $filename

#         ((TEST_RUN++))
#     done
# done

echo Working in $WORK_DIR
echo Output file: $(echo $WORK_DIR | sed 's/\/lustre\/hdahpc\/leichhor/lustre/')/results.csv

echo Now following the result file $WORK_DIR/results.csv:
tail -f $WORK_DIR/results.csv
