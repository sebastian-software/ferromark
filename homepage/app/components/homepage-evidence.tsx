import { useState } from "ardo/react";
import { ArdoLink as Link } from "ardo/runtime";
import {
  MonoBarChart,
  type MonoBarRow,
  MonoSegmented,
  MonoTableView,
} from "ferramenta-family/monografie";

import {
  type Agreement,
  agreementRows,
  conformanceDate,
  featured,
  libraryFigure,
  measured,
  platformLabel,
  platforms,
  type Runtime,
  suiteLabels,
} from "./homepage-data";

type BarRow = { differs?: boolean } & MonoBarRow;

const runtimes: Runtime[] = ["Native", "Node.js"];
const featuredIds = (runtime: Runtime) => (runtime === "Native" ? featured.native : featured.node);
const tableIds = [...featured.native, ...featured.node];
// The legend's example is the first bar the chart opens with.
const example = libraryFigure(platforms[0].id, featured.native[0]).fresh.toFixed(2);

function rows(platform: string, runtime: Runtime): BarRow[] {
  return [
    {
      id: "ferromark",
      label: "Ferromark",
      sub: runtime === "Native" ? "Rust" : "Node.js binding",
      value: 1,
      own: true,
    },
    ...featuredIds(runtime).map((id) => {
      const entry = libraryFigure(platform, id);
      return {
        id,
        label: `${entry.label}${entry.differs ? "*" : ""}`,
        sub: entry.backend,
        value: entry.fresh,
        differs: entry.differs,
      };
    }),
  ];
}

function BenchmarkTable() {
  const [linux, mac] = ["linux-x86-64", "macos-arm64"];
  return (
    <MonoTableView>
      <table className="mono-table">
        <thead>
          <tr>
            <th scope="col">Library</th>
            <th scope="col">Runtime</th>
            <th scope="col" className="mono-num">
              Linux fresh
            </th>
            <th scope="col" className="mono-num">
              macOS fresh
            </th>
            <th scope="col" className="mono-num">
              Linux reuse
            </th>
            <th scope="col" className="mono-num">
              macOS reuse
            </th>
          </tr>
        </thead>
        <tbody>
          {tableIds.map((id) => {
            const onLinux = libraryFigure(linux, id);
            const onMac = libraryFigure(mac, id);
            return (
              <tr key={id}>
                <td>
                  {onLinux.label}
                  {onLinux.differs ? "*" : ""}
                </td>
                <td>{onLinux.runtime}</td>
                <td className="mono-num">{`${onLinux.fresh.toFixed(2)}×`}</td>
                <td className="mono-num">{`${onMac.fresh.toFixed(2)}×`}</td>
                <td className="mono-num">{`${onLinux.reuse.toFixed(2)}×`}</td>
                <td className="mono-num">{`${onMac.reuse.toFixed(2)}×`}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </MonoTableView>
  );
}

const after = (
  <>
    <p>
      Ferromark led all twenty measured libraries; the chart shows the best known.{" "}
      <Link to="/guide/benchmarks">All twenty libraries, methods and raw data</Link>
    </p>
    <BenchmarkTable />
  </>
);

/* The vCPU counts are the Blacksmith runners' from the benchmark guide; homepage-data
   checks that every figure comes from that one campaign. */
const conditions = (
  <>
    <h3>How this was measured</h3>
    <p>
      {measured.date} on Blacksmith VMs: Linux x86-64 on AMD EPYC with 4 vCPU, macOS arm64 with 6
      vCPU. Three independent rounds, fresh calls.
    </p>
    <p>
      All {measured.documents} frozen documents count, including documents where a library&rsquo;s
      output differs (marked *).
    </p>
    <p>Node.js figures include native binding overhead.</p>
    <p>Syntax and API contracts differ; your documents and enabled features change the result.</p>
  </>
);

function describer(machine: string, runtime: Runtime) {
  return (row: BarRow, cut: boolean) =>
    row.own === true ? (
      <>
        <b>Ferromark</b> is the baseline: 1× on {machine}.
      </>
    ) : (
      <>
        <b>{row.label.replace("*", "")}</b> needed {row.value.toFixed(2)}× as long as Ferromark for
        the same {measured.documents} documents on {machine} ({runtime}, fresh calls).
        {cut ? " The bar is cut at 12×." : ""}
        {row.differs === true ? " Some of its outputs differ." : ""}
      </>
    );
}

/** Time for the same work: Ferromark and the best-known alternatives, per machine and runtime. */
export function SpeedChart() {
  const [platform, setPlatform] = useState(platforms[0].id);
  const [runtime, setRuntime] = useState<Runtime>("Native");
  const machine = platformLabel(platform);
  const toggles = (
    <>
      <MonoSegmented
        label="Machine"
        value={platform}
        onChange={setPlatform}
        options={platforms.map((entry) => ({ value: entry.id, label: entry.label }))}
      />
      <MonoSegmented
        label="Runtime"
        value={runtime}
        onChange={(value) => {
          setRuntime(value === "Node.js" ? "Node.js" : "Native");
        }}
        options={runtimes.map((entry) => ({ value: entry, label: entry }))}
      />
    </>
  );
  return (
    <div className="mono-wide">
      <MonoBarChart
        label={`Time for the same work, ${machine}, ${runtime}`}
        title="Time for the same work"
        description={`Ferromark = 1×. A bar of ${example}× means that library needed ${example} times as long.`}
        toggles={toggles}
        rows={rows(platform, runtime)}
        scale={12}
        ticks={[1, 2, 4, 6, 8, 10, 12]}
        describe={describer(machine, runtime)}
        after={after}
        conditions={conditions}
      />
    </div>
  );
}

function Meter({ value }: { value: Agreement }) {
  if (value === null) {
    return (
      <div className="fm-meter">
        <span className="fm-meter-none">not offered</span>
      </div>
    );
  }
  return (
    <div className="fm-meter">
      <span className="fm-meter-track">
        <i style={{ width: `${(value.passed / value.total) * 100}%` }} />
      </span>
      <b>{`${value.passed} / ${value.total}`}</b>
    </div>
  );
}

/** How the best-known libraries agree with the specifications, Ferromark first. */
export function AgreementTable() {
  const first = agreementRows[0];
  return (
    <div className="mono-wide fm-agree">
      <h3>How well-known libraries agree with the specification</h3>
      <p className="fm-agree-sub">
        Measured {conformanceDate} with each library&rsquo;s own specification settings.{" "}
        <Link to="/guide/correctness">Conformance methodology</Link>
      </p>
      <table>
        <thead>
          <tr>
            <th scope="col" style={{ width: "26%" }}>
              Library
            </th>
            <th scope="col" style={{ width: "37%" }}>
              {suiteLabels.commonmark}, {first.commonmark?.total} examples
            </th>
            <th scope="col">
              {suiteLabels.gfm}, {first.gfm?.total} examples
            </th>
          </tr>
        </thead>
        <tbody>
          {agreementRows.map((row) => (
            <tr key={row.id} className={row.own ? "fm-own" : undefined}>
              <td>
                {row.label}
                <small>{row.runtime}</small>
              </td>
              <td>
                <Meter value={row.commonmark} />
              </td>
              <td>
                <Meter value={row.gfm} />
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
