#!/usr/bin/env bash

INITIAL_DIR=$(pwd)

# set -x

if test -z "$1"; then
    echo "No output file specified."
    exit 1
fi

filename=$(readlink -f $1)

if test -e "$filename"; then
    echo "Output file $filename already exists. Please remove it before continuing."
    exit 1
fi

if ! test -e "./runBenchmark.sh"; then
    echo "./runBenchmark.sh is missing or not executable"
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
cp $INITIAL_DIR/runBenchmark.sh .

chmod a+x $WORK_DIR/runBenchmark.sh
output_filename=$WORK_DIR/results.csv

RUNS=30

echo "implementation,name,batch,run,nodes,tasks_per_node,tasks,cpus,entries,total_ram,ram_per_task,reading_the_input,dividing_the_input_into_buckets,sending_to_workers,writing_to_disk,receiving_from_workers,fetching_time_from_workers,receiving_on_worker,sorting_on_worker,sending_to_manager,duration" >$output_filename

min() {
    printf "%s\n" "$1" "$2" | sort -n | head -n1
}

max() {
    printf "%s\n" "$1" "$2" | sort -nr | head -n1
}

get-file-with-entries() {
    case $1 in

    1024)
        echo -n "/lustre/hdahpc/leichhor/sort1K.data"
        ;;

    4096)
        echo -n "/lustre/hdahpc/leichhor/sort4K.data"
        ;;

    16384)
        echo -n "/lustre/hdahpc/leichhor/sort16K.data"
        ;;

    65536)
        echo -n "/lustre/hdahpc/leichhor/sort64K.data"
        ;;

    262144)
        echo -n "/lustre/hdahpc/leichhor/sort256K.data"
        ;;

    1048576)
        echo -n "/lustre/hdahpc/datasets/Sorting/sort1M.data"
        ;;

    4194304)
        echo -n "/lustre/hdahpc/leichhor/sort4M.data"
        ;;

    16777216)
        echo -n "/lustre/hdahpc/datasets/Sorting/sort16M.data"
        ;;

    67108864)
        echo -n "/lustre/hdahpc/datasets/Sorting/sort64M.data"
        ;;

    268435456)
        echo -n "/lustre/hdahpc/datasets/Sorting/sort256M.data"
        ;;

    1073741824)
        echo -n "/lustre/hdahpc/datasets/Sorting/sort1G.data"
        ;;

    *)
        echo "Unsupported number of entries" >&2
        exit 1
        ;;
    esac
}

get-timeouts-from-entries() {
    case $1 in

    1024)
        echo -n "0-00:05:00"
        ;;

    4096)
        echo -n "0-00:05:00"
        ;;

    16384)
        echo -n "0-00:10:00"
        ;;

    65536)
        echo -n "0-00:10:00"
        ;;

    262144)
        echo -n "0-00:10:00"
        ;;

    1048576)
        echo -n "0-00:20:00"
        ;;

    4194304)
        echo -n "0-00:20:00"
        ;;

    16777216)
        echo -n "0-00:20:00"
        ;;

    67108864)
        echo -n "0-01:00:00"
        ;;

    268435456)
        echo -n "0-01:30:00"
        ;;

    1073741824)
        echo -n "0-03:30:00"
        ;;

    *)
        echo "Unsupported number of entries" >&2
        exit 1
        ;;
    esac
}

# in megabytes across all workers
# Currently double the total dataset, but min 4G, because I am too lazy to calculate the exact amount
get-file-size() {
    ENTRIES=$1
    expr $ENTRIES
}

# in megabytes across all workers
# Currently double the total dataset, but min 4G, because I am too lazy to calculate the exact amount
get-multi-requirements() {
    ENTRIES=$1
    TASKS=$2
    SIZE=$ENTRIES

    WORKER_SIZE=$(expr 1000 + $(expr $SIZE \* 2 / \( $TASKS - 1 \) / 10000))

    # min 4000 $(expr $SIZE \* 4 / 10000)
    max $(expr 1000 \* $TASKS) $(expr $WORKER_SIZE \* $TASKS)
}

get-dumb-requirements() {
    ENTRIES=$1
    TASKS=$2
    SIZE=$ENTRIES

    max 2000 $(expr $SIZE \* 2 / 10000)
}

run-benchmark() {
    NUM_NODES=$1
    NUM_TASKS_PER_NODE=$2
    ENTRIES=$3
    OUT_FILE=$4
    RUN_ID=$5
    EXECUTABLE=$6
    MODE=$7

    IN_FILE=$(get-file-with-entries $ENTRIES)
    NUM_TASKS=$(expr $NUM_NODES \* $NUM_TASKS_PER_NODE)

    TOTAL_RAM=0
    NUM_CPUS=0

    case $MODE in
    multi)
        TOTAL_RAM=$(get-multi-requirements $ENTRIES $NUM_TASKS)
        NUM_CPUS=4
        ;;
    single)
        TOTAL_RAM=$(get-multi-requirements $ENTRIES $NUM_TASKS)
        NUM_CPUS=1
        ;;
    dumb)
        TOTAL_RAM=$(get-dumb-requirements $ENTRIES $NUM_TASKS)
        NUM_CPUS=1
        ;;
    *)
        echo "Unsupported mode" >&2
        exit 1
        ;;
    esac

    SUFFIX="-$(expr $RUN_ID % 8)"
    # case $ENTRIES in
    # 1073741824)
    #     SUFFIX="-$(expr $RUN_ID % 16)"
    #     ;;
    # 268435456)
    #     TOTAL_RAM=$(get-multi-requirements $ENTRIES $NUM_TASKS)
    #     SUFFIX="-$(expr $RUN_ID % 8)"
    #     ;;
    # *)
    #     SUFFIX="-$(expr $RUN_ID % 4)"
    #     ;;
    # esac

    RAM_PER_TASK=$(expr $TOTAL_RAM / $NUM_TASKS)
    RAM_PER_CPU=$(expr $(expr $RAM_PER_TASK) / $NUM_CPUS)

    JOBNAME="sort-$ENTRIES-entries-$SUFFIX"
    TIMEOUT=$(get-timeouts-from-entries $ENTRIES)

    # set -x
    sbatch --nice=200 --dependency=singleton --time=${TIMEOUT} --partition=main --job-name=$JOBNAME --nodes=${NUM_NODES} --ntasks-per-node=${NUM_TASKS_PER_NODE} --cpus-per-task=${NUM_CPUS} --mem-per-cpu=${RAM_PER_CPU}M --threads-per-core=1 --error=${WORK_DIR}/%j.err -- ./runBenchmark.sh $NUM_NODES $NUM_TASKS_PER_NODE $ENTRIES $TOTAL_RAM $IN_FILE $OUT_FILE $RUN_ID $EXECUTABLE $MODE
    # set +x
}

# run-benchmark 4 4 1000 $output_filename 1 $WORK_DIR/rust-sorting

# for ENTRIES in 100 1000 1000000 16000000 64000000 256000000 1000000000 ; do
# for ENTRIES in 100 1000 1000000; do
# for ENTRIES in 1000000 16000000; do

# sizes=( 1024 4096 16384 65536 262144 1048576 4194304 16777216 67108864 268435456 1073741824 )
sizes=(1024 4096 16384 65536 262144 1048576 4194304 16777216 67108864 268435456 1073741824)

for ITERATION in $(seq 1 4); do
    for NODES in 1; do
        for ENTRIES in "${sizes[@]}"; do
            RUNN=$((RUN++))
            run-benchmark $NODES 1 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting dumb
            RUNN=$((RUN++))
            run-benchmark $NODES 2 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting multi
            RUNN=$((RUN++))
            run-benchmark $NODES 4 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting multi
            RUNN=$((RUN++))
            run-benchmark $NODES 8 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting multi
            RUNN=$((RUN++))
            run-benchmark $NODES 16 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting multi
            RUNN=$((RUN++))
            run-benchmark $NODES 32 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting multi
            RUNN=$((RUN++))
            run-benchmark $NODES 2 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
            RUNN=$((RUN++))
            run-benchmark $NODES 4 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
            RUNN=$((RUN++))
            run-benchmark $NODES 8 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
            RUNN=$((RUN++))
            run-benchmark $NODES 16 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
            RUNN=$((RUN++))
            run-benchmark $NODES 32 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
            RUNN=$((RUN++))
            run-benchmark $NODES 64 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
            RUNN=$((RUN++))
            run-benchmark $NODES 128 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
        done
    done
    # for NODES in 2 4; do
    #     for ENTRIES in "${sizes[@]}"; do
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 1 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting multi
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 2 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting multi
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 4 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting multi
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 8 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting multi
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 16 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting multi
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 32 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting multi
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 1 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 2 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 4 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 8 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 16 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 32 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 64 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 128 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
    #     done
    # done
    # for NODES in 8; do
    #     for ENTRIES in "${sizes[@]}"; do
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 1 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting multi
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 2 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting multi
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 4 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting multi
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 8 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting multi
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 16 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting multi
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 32 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting multi
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 1 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 2 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 4 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 8 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 16 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 32 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting multi
    #     done
    # done
    # for NODES in 16; do
    #     for ENTRIES in "${sizes[@]}"; do
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 1 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting multi
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 2 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting multi
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 4 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting multi
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 8 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting multi
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 16 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting multi
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 1 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 2 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 4 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 8 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 16 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
    #     done
    # done
    # for NODES in 32; do
    #     for ENTRIES in "${sizes[@]}"; do
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 1 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting multi
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 2 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting multi
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 4 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting multi
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 8 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting multi
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 1 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 2 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 4 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
    #         RUNN=$((RUN++))
    #         run-benchmark $NODES 8 $ENTRIES $output_filename $RUNN $WORK_DIR/rust-sorting single
    #     done
    # done
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
