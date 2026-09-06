import benchmarks from "../data/benchmarks.json"

export { benchmarks }

export type BenchmarkTableData = (typeof benchmarks.tables)[number]
export type BenchmarkRow = BenchmarkTableData["rows"][number]

/** The measured run behind every published figure, formatted as one line. */
export const benchmarkEnvironment = `${benchmarks.run.machine}, ${benchmarks.run.os}, rustc ${benchmarks.run.rustc}, ${benchmarks.run.date}`

/** Locked competitor versions, formatted as one line. */
export const benchmarkCompetitors = `pulldown-cmark ${benchmarks.competitors["pulldown-cmark"]}, comrak ${benchmarks.competitors.comrak}, md4c @ ${benchmarks.competitors.md4c}`

export function benchmarkTable(id: string): BenchmarkTableData {
  const table = benchmarks.tables.find((candidate) => candidate.id === id)
  if (!table) {
    throw new Error(`Unknown benchmark table: ${id}`)
  }
  return table
}

export function benchmarkThroughput(tableId: string, parser: string): string {
  const row = benchmarkTable(tableId).rows.find((candidate) => candidate.parser === parser)
  if (!row) {
    throw new Error(`Unknown parser ${parser} in benchmark table ${tableId}`)
  }
  return row.throughput
}

/** The single figure the landing page leads with. */
export const headlineThroughput = benchmarkThroughput(benchmarks.headline.table, benchmarks.headline.parser)

export function BenchmarkTable({ id, variant = "prose" }: { id: string; variant?: "prose" | "panel" }) {
  const table = benchmarkTable(id)
  const isPanel = variant === "panel"
  const element = (
    <table>
      <thead>
        <tr>
          <th>Parser</th>
          <th>Throughput</th>
          <th>vs ferromark</th>
        </tr>
      </thead>
      <tbody>
        {table.rows.map((row) => {
          const leader = row.parser === benchmarks.headline.parser
          const cell = (value: string) => (leader && !isPanel ? <strong>{value}</strong> : value)
          return (
            <tr key={row.parser} className={leader ? "is-highlight" : undefined}>
              <td>{cell(row.parser)}</td>
              <td>{cell(row.throughput)}</td>
              <td>{cell(row.ratio)}</td>
            </tr>
          )
        })}
      </tbody>
    </table>
  )

  return isPanel ? <div className="benchmark-table">{element}</div> : element
}
