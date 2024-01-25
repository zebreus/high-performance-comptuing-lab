#!/usr/bin/env bash
#SBATCH --threads-per-core=1
#SBATCH --constraint="gold6248r"
#SBATCH --time=02:00:00          # total run time limit (HH:MM:SS)

set -x

NUM_NODES=$1
OUT_FILE=$2
EXECUTABLES_DIR=$3
RUN_ID=$SLURM_JOB_ID

srun --nodes 1 --ntasks=1 -- echo This node seems to be working || exit 7
srun --nodes "$NUM_NODES" --ntasks=1 -- echo All nodes seem to be working || exit 8

filename=$(mktemp kickoff.XXX --tmpdir)

{
    if [[ $NUM_NODES = "1" ]]; then
        for THREADS in 1 2 4 8 16 24 32 40 48 96; do
            # Run a avx512 fake random benchmark
            echo -ne "lgca-10000,${SLURMD_NODENAME},avx512,fake,$RUN_ID,1,48,1,"
            srun --nodes 1 --ntasks-per-node=1 --cpu-bind=core --cpus-per-task=48 -- "${EXECUTABLES_DIR}/lgca-10000" --framerate 0 --threads $THREADS --height 10000 --boxx 2500 --rounds 1000 | grep -v "Singularity container"

            # Run a non avx512 fake random benchmark
            echo -ne "lgca-10000-avx2,${SLURMD_NODENAME},avx2,fake,$RUN_ID,1,48,1,"
            srun --nodes 1 --ntasks-per-node=1 --cpu-bind=core --cpus-per-task=48 -- "${EXECUTABLES_DIR}/lgca-10000-avx2" --framerate 0 --threads $THREADS --height 10000 --boxx 2500 --rounds 1000 | grep -v "Singularity container"

            # Run a avx512 real random benchmark
            echo -ne "lgca-10000-real,${SLURMD_NODENAME},avx512,fake,$RUN_ID,1,48,1,"
            srun --nodes 1 --ntasks-per-node=1 --cpu-bind=core --cpus-per-task=48 -- "${EXECUTABLES_DIR}/lgca-10000-real" --framerate 0 --threads $THREADS --height 10000 --boxx 2500 --rounds 1000 | grep -v "Singularity container"
        done
    fi

    # Run multiple nodes with max threads on each node
    echo -ne "lgca-100,${SLURMD_NODENAME},avx512,fake,$RUN_ID,$NUM_NODES,48,1,"
    srun --nodes "$NUM_NODES" --exclusive --ntasks="$NUM_NODES" --ntasks-per-node=1 --cpu-bind=core --cpus-per-task=48 -- "${EXECUTABLES_DIR}/lgca-100" --framerate 0 --threads 48 --height 100 --boxx 25 --rounds 1000 | grep -v "Singularity container"
    echo -ne "lgca-1000,${SLURMD_NODENAME},avx512,fake,$RUN_ID,$NUM_NODES,48,1,"
    srun --nodes "$NUM_NODES" --exclusive --ntasks="$NUM_NODES" --ntasks-per-node=1 --cpu-bind=core --cpus-per-task=48 -- "${EXECUTABLES_DIR}/lgca-1000" --framerate 0 --threads 48 --height 1000 --boxx 250 --rounds 1000 | grep -v "Singularity container"
    echo -ne "lgca-10000,${SLURMD_NODENAME},avx512,fake,$RUN_ID,$NUM_NODES,48,1,"
    srun --nodes "$NUM_NODES" --exclusive --ntasks="$NUM_NODES" --ntasks-per-node=1 --cpu-bind=core --cpus-per-task=48 -- "${EXECUTABLES_DIR}/lgca-10000" --framerate 0 --threads 48 --height 10000 --boxx 2500 --rounds 1000 | grep -v "Singularity container"

    # Run different combinations of threads and ranks
    echo -ne "lgca-10000,${SLURMD_NODENAME},avx512,fake,$RUN_ID,$NUM_NODES,24,2,"
    srun --nodes "$NUM_NODES" --exclusive --ntasks="$(expr $NUM_NODES \* 2)" --ntasks-per-node=2 --cpu-bind=core --cpus-per-task=24 -- "${EXECUTABLES_DIR}/lgca-10000" --framerate 0 --threads 24 --height 10000 --boxx 2500 --rounds 1000 | grep -v "Singularity container"
    echo -ne "lgca-10000,${SLURMD_NODENAME},avx512,fake,$RUN_ID,$NUM_NODES,16,3,"
    srun --nodes "$NUM_NODES" --exclusive --ntasks="$(expr $NUM_NODES \* 3)" --ntasks-per-node=3 --cpu-bind=core --cpus-per-task=16 -- "${EXECUTABLES_DIR}/lgca-10000" --framerate 0 --threads 16 --height 10000 --boxx 2500 --rounds 1000 | grep -v "Singularity container"
    echo -ne "lgca-10000,${SLURMD_NODENAME},avx512,fake,$RUN_ID,$NUM_NODES,12,4,"
    srun --nodes "$NUM_NODES" --exclusive --ntasks="$(expr $NUM_NODES \* 4)" --ntasks-per-node=4 --cpu-bind=core --cpus-per-task=12 -- "${EXECUTABLES_DIR}/lgca-10000" --framerate 0 --threads 12 --height 10000 --boxx 2500 --rounds 1000 | grep -v "Singularity container"
    echo -ne "lgca-10000,${SLURMD_NODENAME},avx512,fake,$RUN_ID,$NUM_NODES,8,6,"
    srun --nodes "$NUM_NODES" --exclusive --ntasks="$(expr $NUM_NODES \* 6)" --ntasks-per-node=6 --cpu-bind=core --cpus-per-task=8 -- "${EXECUTABLES_DIR}/lgca-10000" --framerate 0 --threads 8 --height 10000 --boxx 2500 --rounds 1000 | grep -v "Singularity container"
    echo -ne "lgca-10000,${SLURMD_NODENAME},avx512,fake,$RUN_ID,$NUM_NODES,6,8,"
    srun --nodes "$NUM_NODES" --exclusive --ntasks="$(expr $NUM_NODES \* 8)" --ntasks-per-node=8 --cpu-bind=core --cpus-per-task=6 -- "${EXECUTABLES_DIR}/lgca-10000" --framerate 0 --threads 6 --height 10000 --boxx 2500 --rounds 1000 | grep -v "Singularity container"
    echo -ne "lgca-10000,${SLURMD_NODENAME},avx512,fake,$RUN_ID,$NUM_NODES,4,12,"
    srun --nodes "$NUM_NODES" --exclusive --ntasks="$(expr $NUM_NODES \* 12)" --ntasks-per-node=12 --cpu-bind=core --cpus-per-task=4 -- "${EXECUTABLES_DIR}/lgca-10000" --framerate 0 --threads 4 --height 10000 --boxx 2500 --rounds 1000 | grep -v "Singularity container"
    echo -ne "lgca-10000,${SLURMD_NODENAME},avx512,fake,$RUN_ID,$NUM_NODES,2,24,"
    srun --nodes "$NUM_NODES" --exclusive --ntasks="$(expr $NUM_NODES \* 24)" --ntasks-per-node=24 --cpu-bind=core --cpus-per-task=2 -- "${EXECUTABLES_DIR}/lgca-10000" --framerate 0 --threads 2 --height 10000 --boxx 2500 --rounds 1000 | grep -v "Singularity container"

    # Big job at the end
    echo -ne "lgca-100000,${SLURMD_NODENAME},avx512,fake,$RUN_ID,$NUM_NODES,24,1,"
    srun --nodes "$NUM_NODES" --exclusive --ntasks="$NUM_NODES" --ntasks-per-node=1 --cpu-bind=core --cpus-per-task=24 -- "${EXECUTABLES_DIR}/lgca-100000" --framerate 0 --threads 24 --height 100000 --boxx 25000 --rounds 1000 | grep -v "Singularity container"

    # This one might also crash
    echo -ne "lgca-10000,${SLURMD_NODENAME},avx512,fake,$RUN_ID,$NUM_NODES,1,48,"
    srun --nodes "$NUM_NODES" --exclusive --ntasks="$(expr $NUM_NODES \* 48)" --ntasks-per-node=48 --cpu-bind=core --cpus-per-task=1 -- "${EXECUTABLES_DIR}/lgca-10000" --framerate 0 --threads 1 --height 10000 --boxx 2500 --rounds 1000 | grep -v "Singularity container"

} >>"$filename"

flock "$OUT_FILE" bash -c "cat $filename | awk NF >> $OUT_FILE"
rm "$filename"
