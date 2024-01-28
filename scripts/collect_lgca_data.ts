import { parse, stringify } from "https://deno.land/std@0.205.0/csv/mod.ts";

const lgcaColumns = [
  "name",
  "slurm_node",
  "vectorization",
  "randomness",
  "run_id",
  "nodes",
  "cpus",
  "tasks_per_node",
  "width",
  "height",
  "rounds",
  "size",
  "threads",
  "core_duration",
  "core_duration_per_cell",
  "top_bottom_duration",
  "top_bottom_duration_per_cell",
  "calculation_duration",
  "calculation_duration_per_cell",
  "communication_duration",
  "total_duration",
  "render_duration",
  "images",
] as const;

const results = parse(await Deno.readTextFile("lgca/results.csv"), {
  skipFirstRow: true,
  columns: lgcaColumns,
});

const coreImplementationComparison = results
  .filter((x) => x["nodes"] == "1")
  .filter((x) => x["width"] == "10000")
  .filter((x) => x["height"] == "10000")
  .filter((x) => x["rounds"] == "1000")
  .filter((x) => x["nodes"] == "1")
  .filter((x) => x["tasks_per_node"] == "1")
  .map((x) => ({
    ...x,
    setup:
      x["randomness"] == "real"
        ? "Real randomness"
        : x["vectorization"] == "avx512"
        ? "512-bit vectorization"
        : "256-bit vectorization",
    cells_per_second:
      (parseFloat(x["width"]) *
        parseFloat(x["height"]) *
        parseFloat(x["rounds"]) *
        parseFloat(x["tasks_per_node"]) *
        parseFloat(x["nodes"])) /
      parseFloat(x["total_duration"]),
  }))
  .map(({ setup, cells_per_second, threads }) => ({
    setup,
    calculation_duration_per_cell: cells_per_second,
    threads: parseInt(threads),
  }));
await Deno.writeTextFile(
  "assets/lgca-core-implementation-comparison.csv",
  stringify(coreImplementationComparison, {
    columns: ["setup", "calculation_duration_per_cell", "threads"],
  })
);

const differentSizesComparison = results
  .filter((x) => x["threads"] == "48")
  .filter((x) => x["tasks_per_node"] == "1")
  .filter((x) => x["nodes"] == "1")
  .filter((x) =>
    ["lgca-100", "lgca-1000", "lgca-10000", "lgca-100000"].includes(x["name"])
  )
  .map((x) => ({
    ...x,
    cells_per_second:
      (parseFloat(x["width"]) *
        parseFloat(x["height"]) *
        parseFloat(x["rounds"]) *
        parseFloat(x["tasks_per_node"]) *
        parseFloat(x["nodes"])) /
      parseFloat(x["total_duration"]),
  }))
  .map(({ width, cells_per_second, nodes }) => ({
    calculation_duration_per_cell: cells_per_second,
    width: parseInt(width),
    nodes: parseInt(nodes),
  }));
await Deno.writeTextFile(
  "assets/lgca-different-sizes-comparison.csv",
  stringify(differentSizesComparison, {
    columns: ["calculation_duration_per_cell", "width", "nodes"],
  })
);

const threadsVsTasksComparison = results
  .filter((x) => parseInt(x["size"]) * parseInt(x["threads"]) == 48)
  .filter((x) => x["width"] == "10000")
  .filter((x) => x["nodes"] == "1")
  .filter((x) => "lgca-10000" == x["name"])
  .map((x) => ({
    ...x,
    cells_per_second:
      (parseFloat(x["width"]) *
        parseFloat(x["height"]) *
        parseFloat(x["rounds"]) *
        parseFloat(x["size"])) /
      parseFloat(x["total_duration"]),
  }))
  .map(({ threads, cells_per_second, size }) => ({
    cells_per_second,
    threads: parseInt(threads),
    size: parseInt(size),
  }));
await Deno.writeTextFile(
  "assets/lgca-threads-vs-size-comparison.csv",
  stringify(threadsVsTasksComparison, {
    columns: ["cells_per_second", "threads", "size"],
  })
);

const multipleNodesComparison = results
  .filter(
    (x) =>
      parseInt(x["size"]) * parseInt(x["threads"]) == parseInt(x["nodes"]) * 48
  )
  .filter((x) => x["width"] == "10000")
  .filter((x) => "lgca-10000" == x["name"])
  .sort((a, b) => parseInt(a["nodes"]) - parseInt(b["nodes"]))
  .map((x) => ({
    ...x,
    cells_per_second:
      (parseFloat(x["width"]) *
        parseFloat(x["height"]) *
        parseFloat(x["rounds"]) *
        parseFloat(x["size"])) /
      parseFloat(x["total_duration"]),
  }))
  .map(({ threads, cells_per_second, size, tasks_per_node, nodes }) => ({
    cells_per_second,
    threads: parseInt(threads),
    size: parseInt(size),
    tasks_per_node: parseInt(tasks_per_node),
    nodes: parseInt(nodes),
  }));
await Deno.writeTextFile(
  "assets/lgca-multiple-nodes-comparison.csv",
  stringify(multipleNodesComparison, {
    columns: ["cells_per_second", "threads", "size", "tasks_per_node", "nodes"],
  })
);
