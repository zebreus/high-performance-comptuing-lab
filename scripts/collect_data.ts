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
