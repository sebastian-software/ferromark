import { startTransition, StrictMode } from "ardo/react";
import { hydrateRoot } from "ardo/react-dom/client";
import { HydratedRouter } from "ardo/router/dom";

startTransition(() => {
  hydrateRoot(
    document,
    <StrictMode>
      <HydratedRouter />
    </StrictMode>,
  );
});
