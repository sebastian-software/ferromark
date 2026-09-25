export function normalizeMdast(value) {
  if (Array.isArray(value)) return value.map(normalizeMdast);

  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value)
        .filter(([key]) => key !== "position")
        .map(([key, child]) => [key, normalizeMdast(child)]),
    );
  }

  return value;
}
