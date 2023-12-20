import { parse, stringify } from "https://deno.land/std@0.205.0/csv/mod.ts";

const sortingColumns = [
  "implementation",
  "name",
  "batch",
  "run",
  "nodes",
  "tasks_per_node",
  "tasks",
  "cpus",
  "entries",
  "total_ram",
  "ram_per_task",
  "reading_the_input",
  "dividing_the_input_into_buckets",
  "sending_to_workers",
  "writing_to_disk",
  "receiving_from_workers",
  "fetching_time_from_workers",
  "receiving_on_worker",
  "sorting_on_worker",
  "sending_to_manager",
  "duration",
] as const;

const results = parse(await Deno.readTextFile("Sorting/results.csv"), {
  skipFirstRow: true,
  columns: sortingColumns,
});

const nonDistributedPerformance = results
  .filter((x) => ["radix-sort", "sort-unstable"].includes(x.implementation))
  .map(
    ({
      implementation,
      duration,
      entries,
      reading_the_input,
      writing_to_disk,
      sorting_on_worker,
    }) => ({
      implementation,
      entries: parseInt(entries),
      reading_the_input: parseFloat(reading_the_input),
      writing_to_disk: parseFloat(writing_to_disk),
      sorting_on_worker: parseFloat(sorting_on_worker),
      duration: parseFloat(duration),
    })
  );
await Deno.writeTextFile(
  "assets/sorting-non-distributed-performance.csv",
  stringify(nonDistributedPerformance, {
    columns: [
      "implementation",
      "duration",
      "entries",
      "reading_the_input",
      "writing_to_disk",
      "sorting_on_worker",
    ],
  })
);

const performanceOnOneNode = results
  .filter((x) => x.nodes === "1")
  .filter((x) => ["1", "2", "32"].includes(x.tasks_per_node))
  .map(({ implementation, tasks, duration, entries }) => ({
    implementation,
    tasks: parseInt(tasks),
    duration: parseFloat(duration),
    entries: parseInt(entries),
  }));
await Deno.writeTextFile(
  "assets/sorting-duration-on-one-node.csv",
  stringify(performanceOnOneNode, {
    columns: ["implementation", "tasks", "duration", "entries"],
  })
);
