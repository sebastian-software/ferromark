import { Link } from "ardo/router";
import { FerraSection } from "ferramenta-family/ferra";

import { BenchmarkComparison, benchmarkHosts, benchmarkLead } from "./benchmark-comparison";

export function HomeMeasurements() {
  return (
    <FerraSection
      id="measurements"
      label="Measurements"
      intro="Performance, with the conditions attached."
      title={benchmarkLead()}
    >
      <p>
        57 documents on macOS and Linux, fresh calls. 2× means twice the throughput. Node.js
        includes binding overhead. Output differences are marked; every timed input contributes to
        the factor.
      </p>
      <div
        className="ferromark-comparison-scroll"
        role="region"
        aria-label="Measured library comparison"
        tabIndex={0}
      >
        <BenchmarkComparison />
      </div>
      <p className="native-benchmark-note">{benchmarkHosts}</p>
      <p>
        <Link to="/guide/benchmarks">Methods and raw data</Link> ·{" "}
        <Link to="/guide/feature-comparison">Features and costs</Link>
      </p>
    </FerraSection>
  );
}
