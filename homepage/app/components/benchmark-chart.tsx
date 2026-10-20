import { useState } from "ardo/react";
import { FerraBars } from "ferramenta-family/ferra";

import {
  nativeBenchmarkBars,
  NativeBenchmarkConditions,
  nativeBenchmarks,
} from "./native-benchmarks";

function RecordedPlatform({
  value,
  onChange,
}: {
  value: string;
  onChange: (value: string) => void;
}) {
  return (
    <label className="ferromark-platform-selector">
      Recorded platform
      <select
        value={value}
        onChange={(event) => {
          onChange(event.target.value);
        }}
      >
        {nativeBenchmarks.platforms.map((item) => (
          <option key={item.id} value={item.id}>
            {item.label}
          </option>
        ))}
      </select>
    </label>
  );
}

/** The archived agreement subsets stay separate from the current complete campaigns. */
export function BenchmarkChart() {
  const [lifecycle, setLifecycle] = useState<"fresh" | "reuse">("fresh");
  const [platformId, setPlatformId] = useState(nativeBenchmarks.platforms[0].id);
  const platform =
    nativeBenchmarks.platforms.find((item) => item.id === platformId) ??
    nativeBenchmarks.platforms[0];
  return (
    <div className="ferromark-benchmark-chart">
      <RecordedPlatform value={platformId} onChange={setPlatformId} />
      <div className="ferromark-benchmark-controls" role="group" aria-label="Measured lifecycle">
        <button
          type="button"
          aria-pressed={lifecycle === "fresh"}
          onClick={() => {
            setLifecycle("fresh");
          }}
        >
          Fresh
        </button>
        <button
          type="button"
          aria-pressed={lifecycle === "reuse"}
          onClick={() => {
            setLifecycle("reuse");
          }}
        >
          Reuse
        </button>
      </div>
      <FerraBars
        caption={`${lifecycle === "fresh" ? "Fresh" : "Reuse"} throughput, relative to Ferromark v2. Higher is faster.`}
        items={nativeBenchmarkBars(lifecycle, platform)}
      >
        <NativeBenchmarkConditions platform={platform} />
      </FerraBars>
    </div>
  );
}
