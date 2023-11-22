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
      "run",
      "duration",
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

const mean = <T extends Record<string, string>>(
  arr: Array<T>,
  key: keyof T,
  groupBy: (keyof T)[]
): Array<T & Record<typeof key, number>> => {
  const groups = arr.reduce((acc, value) => {
    const entryKey = Object.entries(value)
      .filter(([key]) => groupBy.includes(key))
      .map(([_, value]) => value)
      .join("-");
    if (!acc[entryKey]) {
      const withoutKey = { ...value };
      acc[entryKey] = { ...withoutKey, [key]: [] };
    }
    acc[entryKey][key].push(value[key]);
    return acc;
  }, {} as Record<string, T & Record<typeof key, Array<T[typeof key]>>>);
  const result = Object.values(groups).map(
    (x): T & Record<typeof key, number> => ({
      ...x,
      [key]:
        x[key].reduce((a, b) => a + Number.parseFloat(b), 0) / x[key].length,
    })
  );
  return result;
};

const transposedOverview = mean(
  resultsVirgoMatrix.filter((x) =>
    ["openmp-transposed", "rayon"].includes(x.name)
  ),
  "duration",
  ["name", "threads", "matrix_size"]
)
  .toSorted(
    (a, b) =>
      Number.parseFloat(a.matrix_size) - Number.parseFloat(b.matrix_size)
  )
  .toSorted(
    (a, b) => Number.parseFloat(a.threads) - Number.parseFloat(b.threads)
  )
  .map(({ name, duration, matrix_size, threads }) => ({
    name,
    duration,
    matrix_size,
    threads,
  }));
await Deno.writeTextFile(
  "assets/transposed_overview.csv",
  stringify(transposedOverview, {
    columns: ["name", "duration", "threads", "matrix_size"],
  })
);

const originalOverview = mean(
  resultsVirgoMatrix.filter((x) =>
    [
      "openmp",
      "rayon-faithful-pairs",
      "rayon-faithful-iterators",
      "rayon-faithful-unsafe",
    ].includes(x.name)
  ),
  "duration",
  ["name", "threads", "matrix_size"]
)
  .toSorted(
    (a, b) =>
      Number.parseFloat(a.matrix_size) - Number.parseFloat(b.matrix_size)
  )
  .toSorted(
    (a, b) => Number.parseFloat(a.threads) - Number.parseFloat(b.threads)
  )
  .map(({ name, duration, matrix_size, threads }) => ({
    name,
    duration,
    matrix_size,
    threads,
  }));
await Deno.writeTextFile(
  "assets/original_overview.csv",
  stringify(originalOverview, {
    columns: ["name", "duration", "threads", "matrix_size"],
  })
);

const transposedHighThreads = resultsVirgoMatrix
  .filter((x) => ["openmp-transposed", "rayon"].includes(x.name))
  .filter((x) => x.threads === "64")
  .toSorted(
    (a, b) =>
      Number.parseFloat(a.matrix_size) - Number.parseFloat(b.matrix_size)
  )
  .map(({ name, duration, matrix_size }) => ({ name, duration, matrix_size }));
await Deno.writeTextFile(
  "assets/transposed-high-threads.csv",
  stringify(transposedHighThreads, {
    columns: ["name", "duration", "matrix_size"],
  })
);

const originalHighThreads = resultsVirgoMatrix
  .filter((x) =>
    [
      "openmp",
      "rayon-faithful-pairs",
      "rayon-faithful-iterators",
      "rayon-faithful-unsafe",
    ].includes(x.name)
  )
  .filter((x) => x.threads === "64")
  .toSorted(
    (a, b) =>
      Number.parseFloat(a.matrix_size) - Number.parseFloat(b.matrix_size)
  )
  .map(({ name, duration, matrix_size }) => ({ name, duration, matrix_size }));
await Deno.writeTextFile(
  "assets/original-high-threads.csv",
  stringify(originalHighThreads, {
    columns: ["name", "duration", "matrix_size"],
  })
);

const algorithmComparisonHighThreads = resultsVirgoMatrix
  .filter((x) =>
    ["rayon", "rayon-faithful-pairs", "rayon-faithful-mutex"].includes(x.name)
  )
  .filter((x) => x.threads === "64")
  .toSorted(
    (a, b) =>
      Number.parseFloat(a.matrix_size) - Number.parseFloat(b.matrix_size)
  )
  .map(({ name, duration, matrix_size }) => ({ name, duration, matrix_size }));
await Deno.writeTextFile(
  "assets/algorithm-comparison-high-threads.csv",
  stringify(algorithmComparisonHighThreads, {
    columns: ["name", "duration", "matrix_size"],
  })
);

const performanceLowThreads = resultsVirgoMatrix
  .filter((x) => x.threads === "1")
  .toSorted(
    (a, b) =>
      Number.parseFloat(a.matrix_size) - Number.parseFloat(b.matrix_size)
  )
  .map(({ name, duration, matrix_size }) => ({ name, duration, matrix_size }));
await Deno.writeTextFile(
  "assets/performance-low-threads.csv",
  stringify(performanceLowThreads, {
    columns: ["name", "duration", "matrix_size"],
  })
);

const performanceMediumThreads = resultsVirgoMatrix
  .filter((x) => x.threads === "8")
  .toSorted(
    (a, b) =>
      Number.parseFloat(a.matrix_size) - Number.parseFloat(b.matrix_size)
  )
  .map(({ name, duration, matrix_size }) => ({ name, duration, matrix_size }));
await Deno.writeTextFile(
  "assets/performance-medium-threads.csv",
  stringify(performanceMediumThreads, {
    columns: ["name", "duration", "matrix_size"],
  })
);

const performanceHighThreads = resultsVirgoMatrix
  .filter((x) => x.threads === "64")
  .toSorted(
    (a, b) =>
      Number.parseFloat(a.matrix_size) - Number.parseFloat(b.matrix_size)
  )
  .map(({ name, duration, matrix_size }) => ({ name, duration, matrix_size }));
await Deno.writeTextFile(
  "assets/performance-high-threads.csv",
  stringify(performanceHighThreads, {
    columns: ["name", "duration", "matrix_size"],
  })
);
