import { parse, stringify } from "https://deno.land/std@0.205.0/csv/mod.ts";

const kickoffColumns = ["name", "threads", "n", "run", "duration"] as const;

const resultsClang = parse(
  await Deno.readTextFile("KickOff/results_clang.csv"),
  {
    skipFirstRow: true,
    columns: kickoffColumns,
  }
);
const resultsGcc = parse(await Deno.readTextFile("KickOff/results_gcc.csv"), {
  skipFirstRow: true,
  columns: kickoffColumns,
});

const resultsVirgo = parse(
  await Deno.readTextFile("KickOff/results_virgo.csv"),
  {
    skipFirstRow: true,
    columns: ["name", "threads", "n", "duration"] as const,
  }
);

const resultsVirgoMatrix = parse(
  await Deno.readTextFile("MatrixMult/results_virgo.csv"),
  {
    skipFirstRow: true,
    columns: [
      "name",
      "threads",
      "matrix_size",
      "duration",
      "run",
      "sum",
    ] as const,
  }
);

const compilerComparison = [
  ...resultsClang.map((x) => ({ ...x, compiler: "clang" })),
  ...resultsGcc.map((x) => ({ ...x, compiler: "gcc" })),
]
  .filter((x) => x.threads === "8")
  .map(({ compiler, duration, n }) => ({ compiler, duration, n }));
await Deno.writeTextFile(
  "assets/compiler-comparison.csv",
  stringify(compilerComparison, {
    columns: ["compiler", "duration", "n"],
  })
);

const threadsAndComplexity = resultsClang
  .filter((x) => x.name === "mpi-pi++")
  .map(({ n, duration, threads }) => ({ n, duration, threads }));
await Deno.writeTextFile(
  "assets/mpi-cpp.csv",
  stringify(threadsAndComplexity, {
    columns: ["n", "duration", "threads"],
  })
);

const implementationComparisonFixedN = resultsClang
  .filter((x) => x.n === "2048")
  .map(({ name, duration, threads }) => ({ name, duration, threads }));
await Deno.writeTextFile(
  "assets/implementation-comparison-fixed-n.csv",
  stringify(implementationComparisonFixedN, {
    columns: ["name", "duration", "threads"],
  })
);

const implementationComparisonFixedThreads = resultsClang
  .filter((x) => x.threads === "8")
  .map(({ name, duration, n }) => ({ name, duration, n }));
await Deno.writeTextFile(
  "assets/implementation-comparison-fixed-threads.csv",
  stringify(implementationComparisonFixedThreads, {
    columns: ["name", "duration", "n"],
  })
);

const mpiCppVirgo = resultsVirgo
  .filter((x) => x.name === "mpi-pi++")
  .toSorted((a, b) => Number.parseFloat(a.n) - Number.parseFloat(b.n))
  .toSorted(
    (a, b) => Number.parseFloat(a.threads) - Number.parseFloat(b.threads)
  )
  .map(({ threads, duration, n }) => ({ threads, duration, n }));
await Deno.writeTextFile(
  "assets/mpi-cpp-virgo.csv",
  stringify(mpiCppVirgo, {
    columns: ["threads", "duration", "n"],
  })
);

const openmpVirgo = resultsVirgo
  .filter((x) => x.name === "openMP-pi")
  .toSorted((a, b) => Number.parseFloat(a.n) - Number.parseFloat(b.n))
  .toSorted(
    (a, b) => Number.parseFloat(a.threads) - Number.parseFloat(b.threads)
  )
  .map(({ threads, duration, n }) => ({ threads, duration, n }));
await Deno.writeTextFile(
  "assets/openmp-virgo.csv",
  stringify(openmpVirgo, {
    columns: ["threads", "duration", "n"],
  })
);

const cppThreadsVirgo = resultsVirgo
  .filter((x) => x.name === "cpp11-pi")
  .toSorted((a, b) => Number.parseFloat(a.n) - Number.parseFloat(b.n))
  .toSorted(
    (a, b) => Number.parseFloat(a.threads) - Number.parseFloat(b.threads)
  )
  .map(({ threads, duration, n }) => ({ threads, duration, n }));
await Deno.writeTextFile(
  "assets/cpp-threads-virgo.csv",
  stringify(cppThreadsVirgo, {
    columns: ["threads", "duration", "n"],
  })
);

const performanceLowThreads = resultsVirgoMatrix
  .filter((x) => x.threads === "1")
  .toSorted((a, b) => Number.parseFloat(a.n) - Number.parseFloat(b.n))
  .map(({ name, duration, n }) => ({ name, duration, n }));
await Deno.writeTextFile(
  "assets/performance-low-threads.csv",
  stringify(performanceLowThreads, {
    columns: ["name", "duration", "matrix_size"],
  })
);

const performanceMediumThreads = resultsVirgoMatrix
  .filter((x) => x.threads === "8")
  .toSorted((a, b) => Number.parseFloat(a.n) - Number.parseFloat(b.n))
  .map(({ name, duration, n }) => ({ name, duration, n }));
await Deno.writeTextFile(
  "assets/performance-medium-threads.csv",
  stringify(performanceMediumThreads, {
    columns: ["name", "duration", "matrix_size"],
  })
);

const performanceHighThreads = resultsVirgoMatrix
  .filter((x) => x.threads === "128")
  .toSorted((a, b) => Number.parseFloat(a.n) - Number.parseFloat(b.n))
  .map(({ name, duration, n }) => ({ name, duration, n }));
await Deno.writeTextFile(
  "assets/performance-high-threads.csv",
  stringify(performanceHighThreads, {
    columns: ["name", "duration", "matrix_size"],
  })
);
