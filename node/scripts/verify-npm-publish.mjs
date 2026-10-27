import { createNativeRelease } from "ferramenta-native/release";

import config from "../native-npm.config.mjs";
export {
  MAX_ATTEMPTS,
  registryPackageUrl,
  registryVersionUrl,
  REQUEST_TIMEOUT_MS,
  RETRY_DELAY_MS,
  verifyNpmPublication,
} from "ferramenta-native/publication";
if (process.argv[1] === import.meta.filename) await createNativeRelease(config).verify();
