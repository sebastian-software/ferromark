import { useId, useState } from "react";
import nativeBenchmarks from "../data/native-benchmarks.json";

export { nativeBenchmarks };

const sourceUrl = (path: string) =>
  `https://github.com/sebastian-software/ferromark/blob/main/${path}`;

export function NativeComparisonTable({ selectedCase }: { selectedCase?: string }) {
  const tables = nativeBenchmarks.tables.filter(
    (table) => !selectedCase || table.case === selectedCase,
  );
  return tables.map((table) => (
    <div key={table.case}>
      {!selectedCase && <h3>Native {table.label}</h3>}
      <p>
        {table.bytes.toLocaleString("en-US")} input bytes. Lower time and higher throughput are
        better.
      </p>
      <div
        className="native-benchmark-table"
        role="region"
        aria-label={`Native engine timings: ${table.label}`}
        tabIndex={0}
      >
        <table>
          <thead>
            <tr>
              <th scope="col">Engine</th>
              <th scope="col">Time / document</th>
              <th scope="col">Throughput</th>
              <th scope="col">Relative speed</th>
            </tr>
          </thead>
          <tbody>
            {table.rows.map((row) => {
              const cell = (value: string) => (row.winner ? <strong>{value}</strong> : value);
              return (
                <tr key={row.id} className={row.winner ? "is-highlight" : undefined}>
                  <th scope="row">
                    <a href={sourceUrl(row.report)}>{cell(row.label)}</a>
                  </th>
                  <td>{cell(row.latency)}</td>
                  <td>{cell(row.throughput)}</td>
                  <td>{cell(row.relativeSpeed)}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
      {table.unmeasured.length > 0 && (
        <p className="native-benchmark-note">
          Not measured for this document: {table.unmeasured.join(", ")}.
        </p>
      )}
    </div>
  ));
}

export function NativeBenchmarkExplorer() {
  const selectorId = useId();
  const [selectedCase, setSelectedCase] = useState("gfm_overlap/features");
  const cases = nativeBenchmarks.tables.map((table) => [table.case, table.label]);
  return (
    <>
      <div className="native-benchmark-selector">
        <label htmlFor={selectorId}>Workload</label>
        <select
          id={selectorId}
          value={selectedCase}
          onChange={(event) => setSelectedCase(event.target.value)}
        >
          {cases.map(([id, label]) => (
            <option key={id} value={id}>
              {label}
            </option>
          ))}
        </select>
      </div>
      <NativeComparisonTable selectedCase={selectedCase} />
      <p className="native-benchmark-note">
        Ferromark: median across reference runs for this document. Relative speed uses this single
        baseline. Bold marks the fastest result; original measurements are linked in the guide.
      </p>
    </>
  );
}

export function NativeComparisonCoverage() {
  return (
    <>
      <div
        className="native-benchmark-table"
        role="region"
        aria-label="Native comparison coverage"
        tabIndex={0}
      >
        <table>
          <thead>
            <tr>
              <th scope="col">Engine</th>
              <th scope="col">Runtime</th>
              <th scope="col">Admitted workloads</th>
              <th scope="col">Spec mismatches</th>
              <th scope="col">Measured source</th>
            </tr>
          </thead>
          <tbody>
            {nativeBenchmarks.engines.map((engine) => (
              <tr key={engine.id}>
                <th scope="row">{engine.label}</th>
                <td>{engine.runtime}</td>
                <td>
                  {engine.eligible}/{engine.total}
                </td>
                <td>
                  {engine.specMismatches}/{engine.specTotal}
                </td>
                <td>
                  <a href={sourceUrl(engine.report)}>{engine.version}</a>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <p>
        Admission checks comparable Markdown work. Spec mismatches are normalized output
        diagnostics, not a conformance certification. Only the workloads in the timing table
        received full measurement runs.
      </p>
      <ul>
        {nativeBenchmarks.engines.map((engine) => (
          <li key={engine.id}>
            <strong>{engine.label}:</strong> {engine.notes}
          </li>
        ))}
      </ul>
    </>
  );
}
