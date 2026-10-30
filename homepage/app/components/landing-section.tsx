import type { ReactNode } from "ardo/react";

import { FerraSoftCard } from "ferramenta-family/ferra";

type SectionProps = {
  id: string;
  title: string;
  intro?: ReactNode;
  note?: ReactNode;
  className?: string;
  children: ReactNode;
};

export function LandingSection({ id, title, intro, note, className = "", children }: SectionProps) {
  return (
    <FerraSoftCard
      as="section"
      id={id}
      aria-labelledby={`${id}-title`}
      className={`ferromark-section ${className}`}
    >
      <header className="ferromark-section-heading">
        <h2 id={`${id}-title`}>{title}</h2>
        {intro !== undefined && intro !== null && <p>{intro}</p>}
        {note !== undefined && note !== null && (
          <div className="ferromark-section-note">{note}</div>
        )}
      </header>
      {children}
    </FerraSoftCard>
  );
}

export function FeatureGrid({
  items,
}: {
  items: ReadonlyArray<{ heading: string; text: string }>;
}) {
  return (
    <div className="ferromark-feature-grid">
      {items.map((item) => (
        <div key={item.heading}>
          <h3>{item.heading}</h3>
          <p>{item.text}</p>
        </div>
      ))}
    </div>
  );
}
