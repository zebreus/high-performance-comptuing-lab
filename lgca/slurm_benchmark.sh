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

if ! test -e "./run_benchmark.sh"; then
    echo "./run_benchmark.sh is missing or not executable"
    exit 1
fi

if ! test -e "./build_binaries.sh"; then
    echo "./build_binaries.sh is missing or not executable"
    exit 1
fi

if test -z "$LUSTRE_HOME"; then
    echo "LUSTRE_HOME is not set"
    exit 1
fi

WORK_DIR=$(mktemp -d --tmpdir=$LUSTRE_HOME lgca-benchmark.XXXX)
mkdir -p "$WORK_DIR"
date >>$WORK_DIR/date.txt

mkdir -p "${WORK_DIR}/executables"
bash ./build_binaries.sh "$WORK_DIR/executables"

cp $INITIAL_DIR/run_benchmark.sh "$WORK_DIR"

cd $WORK_DIR

chmod a+x $WORK_DIR/run_benchmark.sh
output_filename=$WORK_DIR/results.csv

echo "name,slurm_node,vectorization,randomness,run_id,nodes,cpus,tasks_per_node,width,height,rounds,size,threads,core_duration,core_duration_per_cell,top_bottom_duration,top_bottom_duration_per_cell,calculation_duration,calculation_duration_per_cell,communication_duration,total_duration,render_duration,images" >$output_filename

run-benchmark() {
    NUM_NODES=$1
    EXECUTABLES_DIR=${WORK_DIR}/executables
    OUT_FILE="$output_filename"
    ITERATION=$2

    JOBNAME="lgca-benchmark-round-$ITERATION"

    TIME="00:45:00"
    if [[ $NUM_NODES = "1" ]]; then
        TIME="02:30:00"
    fi

    # set -x
    sbatch -p main --exclusive --time="${TIME}" --job-name="$JOBNAME" --constraint="gold6248r" --nodes=${NUM_NODES} --cores-per-socket=24 --ntasks-per-node=1 --cpus-per-task=48 --threads-per-core=1 --error=${WORK_DIR}/%j.err -- ./run_benchmark.sh $NUM_NODES $OUT_FILE $EXECUTABLES_DIR
    # set +x
}

for ITERATION in $(seq 1 5); do
    for NODES in 1 2 4 8 16; do
        run-benchmark $NODES "$ITERATION"
    done
done

echo Working in $WORK_DIR
echo Output file: $(echo $WORK_DIR | sed 's/\/lustre\/hdahpc\/leichhor/lustre/')/results.csv

echo Now following the result file $WORK_DIR/results.csv:
tail -f $WORK_DIR/results.csv
