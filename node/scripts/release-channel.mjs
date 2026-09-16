export function releaseChannel(version) {
  const number = "(?:0|[1-9]\\d*)";
  const stable = `${number}\\.${number}\\.${number}`;
  if (new RegExp(`^${stable}$`).test(version)) return { tag: "latest", prerelease: false };
  if (new RegExp(`^${stable}-rc\\.${number}$`).test(version))
    return { tag: "next", prerelease: true };
  throw new Error(`Release version must be stable or an rc: ${version}`);
}
