import { samples } from "./homepage-data";

/* Six thin plates in sequence, drawn in the instrument's materials. The first
   and the last carry the real basic sample: its source and its HTML. */
const W = 138;
const D = 96;
const T = 7;
const DX = 58;
const DY = 44;
const STEP = 204;
const Y0 = 62;

const stages = [
  ["Source bytes", "The Markdown text as written"],
  ["Parser", "Specialized byte scans, SIMD where offered"],
  ["Arena", "One block of memory for every node"],
  ["Tree", "Nodes keep byte offsets into the source"],
  ["Renderer", "HTML out; hooks for code and diagrams"],
  ["HTML", "An owned string, or appended to yours"],
] as const;

const lines = (text: string) => text.split("\n").filter((line) => line.trim() !== "");
const clip = (line: string, length: number) =>
  line.length > length ? `${line.slice(0, length - 1)}…` : line;
const sourceLines = lines(samples.basic.markdown).slice(0, 4);
const outputLines = lines(samples.basic.html).slice(0, 4);

const scanHits = new Set(["0/0", "1/5", "2/2"]);
const treeNodes = [
  [68, 14],
  [34, 46],
  [102, 46],
  [18, 80],
  [52, 80],
  [102, 80],
] as const;
const treeEdges = [
  [0, 1],
  [0, 2],
  [1, 3],
  [1, 4],
  [2, 5],
] as const;

function SourcePlate() {
  return sourceLines.map((line, index) => (
    <text key={line} className="fm-iso-code" x={10} y={22 + index * 20} fontSize={9.5}>
      {clip(line, 22)}
    </text>
  ));
}

function ParserPlate() {
  return [0, 1, 2].flatMap((row) =>
    Array.from({ length: 9 }, (_, column) => (
      <rect
        key={`${row}/${column}`}
        className={scanHits.has(`${row}/${column}`) ? "fm-iso-hit" : "fm-iso-cell"}
        x={10 + column * 13.4}
        y={12 + row * 27}
        width={10}
        height={20}
      />
    )),
  );
}

function ArenaPlate() {
  return Array.from({ length: 36 }, (_, index) => (
    <rect
      key={index}
      className={index < 19 ? "fm-iso-filled" : "fm-iso-slot"}
      x={8 + (index % 9) * 13.6}
      y={8 + Math.floor(index / 9) * 21}
      width={11.5}
      height={17}
    />
  ));
}

function TreePlate() {
  return (
    <>
      {treeEdges.map(([from, to]) => (
        <line
          key={`${from}-${to}`}
          className="fm-iso-edge"
          x1={treeNodes[from][0]}
          y1={treeNodes[from][1]}
          x2={treeNodes[to][0]}
          y2={treeNodes[to][1]}
        />
      ))}
      {treeNodes.map(([x, y], index) => (
        <circle
          key={`${x}/${y}`}
          className={index === 0 ? "fm-iso-root" : "fm-iso-node"}
          cx={x}
          cy={y}
          r={index === 0 ? 8 : 6}
        />
      ))}
    </>
  );
}

function RendererPlate() {
  return ["code", "highlight", "diagram"].map((hook, index) => (
    <g key={hook}>
      <rect className="fm-iso-cell" x={10} y={10 + index * 28} width={58} height={18} />
      <text className="fm-iso-code" x={15} y={23 + index * 28} fontSize={9}>
        {hook}
      </text>
      <line className="fm-iso-wire" x1={76} y1={19 + index * 28} x2={126} y2={19 + index * 28} />
    </g>
  ));
}

function HtmlPlate() {
  return outputLines.map((line, index) => (
    <text key={line} className="fm-iso-code" x={8} y={22 + index * 20} fontSize={8.6}>
      {clip(line, 25)}
    </text>
  ));
}

const plates = [SourcePlate, ParserPlate, ArenaPlate, TreePlate, RendererPlate, HtmlPlate];

// One drawing per page, so the references inside the SVG can keep plain IDs.
const ids = {
  title: "fm-iso-title",
  description: "fm-iso-description",
  steel: "fm-iso-steel",
  oxide: "fm-iso-oxide",
  shadow: "fm-iso-shadow",
};

function Definitions() {
  return (
    <defs>
      <linearGradient id={ids.steel} x1={0} y1={0} x2={1} y2={1}>
        <stop offset={0} className="fm-iso-steel-1" />
        <stop offset={0.55} className="fm-iso-steel-2" />
        <stop offset={1} className="fm-iso-steel-3" />
      </linearGradient>
      <linearGradient id={ids.oxide} x1={0} y1={0} x2={1} y2={1}>
        <stop offset={0} className="fm-iso-oxide-1" />
        <stop offset={0.6} className="fm-iso-oxide-2" />
        <stop offset={1} className="fm-iso-oxide-3" />
      </linearGradient>
      <filter id={ids.shadow} x="-20%" y="-20%" width="140%" height="160%">
        <feDropShadow dx={0} dy={10} stdDeviation={8} floodColor="#26231f" floodOpacity={0.12} />
      </filter>
    </defs>
  );
}

function Plate({ stage }: { stage: number }) {
  const [name, detail] = stages[stage];
  const Content = plates[stage];
  const x0 = 18 + stage * STEP;
  const hot = stage === 2 || stage === 3;
  return (
    <g className={hot ? "fm-iso-plate fm-iso-hot" : "fm-iso-plate"}>
      <g filter={`url(#${ids.shadow})`}>
        <path
          className="fm-iso-side"
          d={`M${x0} ${Y0} H${x0 + W} L${x0 + W + DX} ${Y0 - DY} V${Y0 - DY + T} L${x0 + W} ${Y0 + T} H${x0} Z`}
        />
        <path
          className="fm-iso-top"
          fill={`url(#${hot ? ids.oxide : ids.steel})`}
          d={`M${x0 + DX} ${Y0 - DY} H${x0 + W + DX} L${x0 + W} ${Y0} H${x0} Z`}
        />
        <path className="fm-iso-edge-light" d={`M${x0} ${Y0 + 0.5} H${x0 + W}`} />
        <g transform={`matrix(1 0 -${DX / D} ${DY / D} ${x0 + DX} ${Y0 - DY})`}>
          <Content />
        </g>
      </g>
      <text className="fm-iso-label" x={x0} y={Y0 + T + 40}>
        {`${stage + 1}  ${name}`}
      </text>
      <foreignObject x={x0} y={Y0 + T + 50} width={184} height={44}>
        <div className="fm-iso-caption">{detail}</div>
      </foreignObject>
    </g>
  );
}

/** Inside the engine: the source, parser, arena, tree, renderer and HTML as thin plates. */
export function EngineDrawing() {
  const bracketStart = 18 + 2 * STEP;
  const bracketEnd = 18 + 3 * STEP + W + DX;
  const bracketY = Y0 + T + 112;
  return (
    <figure className="mono-full fm-drawing">
      <svg viewBox="0 0 1240 290" role="img" aria-labelledby={`${ids.title} ${ids.description}`}>
        <title id={ids.title}>The stages inside Ferromark</title>
        <desc id={ids.description}>
          Six thin plates in sequence: the source bytes, the parser with its byte scans, the arena
          that holds every node, the document tree with source spans, the renderer with its hooks,
          and the HTML output. The arena and the tree are marked in oxide.
        </desc>
        <Definitions />
        {stages.map(([name], stage) => (
          <Plate key={name} stage={stage} />
        ))}
        <path
          className="fm-iso-bracket"
          d={`M${bracketStart} ${bracketY - 8} V${bracketY} H${bracketEnd} V${bracketY - 8}`}
        />
        <text
          className="fm-iso-note"
          x={(bracketStart + bracketEnd) / 2}
          y={bracketY + 22}
          textAnchor="middle"
        >
          filled, used and released at once
        </text>
      </svg>
    </figure>
  );
}
