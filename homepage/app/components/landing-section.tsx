import type { ReactNode } from "ardo/react";

import { FerraArticle } from "ferramenta-family/ferra";

export { FerraFeatureGrid as FeatureGrid } from "ferramenta-family/ferra";

export function LandingSection({
  intro,
  note,
  ...props
}: {
  id: string;
  title: string;
  intro?: ReactNode;
  note?: ReactNode;
  children: ReactNode;
  className?: string;
}) {
  return <FerraArticle {...props} introduction={intro} meta={note} />;
}
