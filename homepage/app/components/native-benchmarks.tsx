import { useId, useState } from "react";
import nativeBenchmarks from "../data/native-benchmarks.json";

export { nativeBenchmarks };

const sourceUrl = (path: string) =>
  `https://github.com/sebastian-software/ferromark/blob/main/${path}`;

export function NativeComparisonTable({ selectedCase }: { selectedCase?: string }) {
  return (
    <div
      className="native-benchmark-table"
      role="region"
      aria-label="Native engine timings"
      tabIndex={0}
    >
      <table>
        <thead>
          <tr>
            <th scope="col">Native pair</th>
            {!selectedCase && <th scope="col">Input</th>}
            <th scope="col">Bytes</th>
            <th scope="col">Ferromark µs</th>
            <th scope="col">Candidate µs</th>
            <th scope="col">Candidate / Ferromark</th>
          </tr>
        </thead>
        <tbody>
          {nativeBenchmarks.engines.flatMap((engine) => {
            const rows = engine.rows.filter((row) => !selectedCase || row.case === selectedCase);
            if (!rows.length) {
              return (
                <tr key={engine.id}>
                  <th scope="row">
                    <a href={sourceUrl(engine.report)}>{engine.label}</a>
                  </th>
                  <td colSpan={4}>
                    Not measured{engine.id === "ox-content" ? " — heading-ID mismatch" : ""}
                  </td>
                </tr>
              );
            }
            return rows.map((row) => (
              <tr key={`${engine.id}/${row.case}`}>
                <th scope="row">
                  <a href={sourceUrl(engine.report)}>{engine.label}</a>
                </th>
                {!selectedCase && <td>{row.label}</td>}
                <td>{row.bytes.toLocaleString("en-US")}</td>
                <td>
                  {row.ferromarkNs <= row.candidateNs ? (
                    <strong>{row.ferromarkTime}</strong>
                  ) : (
                    row.ferromarkTime
                  )}
                </td>
                <td>
                  {row.candidateNs <= row.ferromarkNs ? (
                    <strong>{row.candidateTime}</strong>
                  ) : (
                    row.candidateTime
                  )}
                </td>
                <td>{row.ratio}</td>
              </tr>
            ));
          })}
        </tbody>
      </table>
    </div>
  );
}

export function NativeBenchmarkExplorer() {
  const selectorId = useId();
  const [selectedCase, setSelectedCase] = useState("gfm_overlap/features");
  const cases = [
    ...new Map(
      nativeBenchmarks.engines.flatMap((engine) =>
        engine.rows.map((row) => [row.case, row.label] as const),
      ),
    ).entries(),
  ];
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
      <p className="native-benchmark-note">{nativeBenchmarks.ratioExplanation}</p>
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
