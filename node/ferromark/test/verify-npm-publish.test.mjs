import assert from "node:assert/strict";
import test from "node:test";

import {
  MAX_ATTEMPTS,
  registryPackageUrl,
  registryVersionUrl,
  verifyNpmPublication,
} from "../../scripts/verify-npm-publish.mjs";

/** A registry response, or HTTP 404 for an `undefined` document. */
const respond = (body) =>
  body === undefined
    ? { ok: false, status: 404, json: () => Promise.resolve({}) }
    : { ok: true, status: 200, json: () => Promise.resolve(body) };

/** A registry that answers with the given version document and dist-tags. */
function registry({ packageName, versionDocument, distTags }) {
  const requests = [];

  return {
    requests,
    fetchImpl(url, options) {
      requests.push({ url, options });
      if (url === registryPackageUrl(packageName)) {
        return Promise.resolve(respond({ name: packageName, "dist-tags": distTags }));
      }
      return Promise.resolve(respond(versionDocument));
    },
  };
}

const sleepImpl = () => Promise.resolve();

test("confirms a published version and the dist-tag that installs it", async () => {
  const version = "2.0.0-rc.2";
  const { fetchImpl, requests } = registry({
    packageName: "ferromark",
    versionDocument: { name: "ferromark", version },
    distTags: { latest: "0.7.0", next: version },
  });

  await verifyNpmPublication({
    packageName: "ferromark",
    version,
    publishResult: "success",
    fetchImpl,
    sleepImpl,
  });

  assert.deepEqual(
    requests.map(({ url }) => url),
    [registryVersionUrl("ferromark", version), registryPackageUrl("ferromark")],
  );
  assert.equal(requests[1].options.headers.accept, "application/vnd.npm.install-v1+json");
});

test("fails when the release channel still points at an older version", async () => {
  const version = "2.0.0";
  const { fetchImpl, requests } = registry({
    packageName: "ferromark",
    versionDocument: { name: "ferromark", version },
    // The stable release publishes to latest, which v1 still holds here.
    distTags: { latest: "0.7.0", next: "2.0.0-rc.2" },
  });

  await assert.rejects(
    verifyNpmPublication({
      packageName: "ferromark",
      version,
      publishResult: "success",
      fetchImpl,
      sleepImpl,
    }),
    (error) => {
      assert.match(error.message, /npm release verification failed/);
      assert.match(error.message, /dist-tag latest/);
      assert.match(error.message, /registry lists ferromark@latest as 0\.7\.0/);
      return true;
    },
  );
  assert.equal(requests.length, MAX_ATTEMPTS * 2);
});

test("fails when a sidecar carries no dist-tag for the channel", async () => {
  const version = "2.0.0-rc.2";
  const { fetchImpl } = registry({
    packageName: "ferromark-linux-x64-gnu",
    versionDocument: { name: "ferromark-linux-x64-gnu", version },
    distTags: { latest: "2.0.0-rc.1" },
  });

  await assert.rejects(
    verifyNpmPublication({
      packageName: "ferromark-linux-x64-gnu",
      version,
      publishResult: "success",
      fetchImpl,
      sleepImpl,
    }),
    /registry lists ferromark-linux-x64-gnu@next as unset/,
  );
});

test("fails when the version is missing and never asks for its dist-tags", async () => {
  const version = "2.0.0-rc.2";
  const { fetchImpl, requests } = registry({
    packageName: "ferromark",
    versionDocument: undefined,
    distTags: { next: version },
  });

  await assert.rejects(
    verifyNpmPublication({
      packageName: "ferromark",
      version,
      publishResult: "success",
      fetchImpl,
      sleepImpl,
    }),
    /registry returned HTTP 404/,
  );
  assert.deepEqual(
    [...new Set(requests.map(({ url }) => url))],
    [registryVersionUrl("ferromark", version)],
  );
});

test("reports a failed publish step together with the registry observation", async () => {
  const version = "2.0.0-rc.2";
  const { fetchImpl } = registry({
    packageName: "ferromark",
    versionDocument: { name: "ferromark", version },
    distTags: { next: version },
  });

  await assert.rejects(
    verifyNpmPublication({
      packageName: "ferromark",
      version,
      publishResult: "failure",
      fetchImpl,
      sleepImpl,
    }),
    /publish-npm concluded failure/,
  );
});

test("rejects versions that no release channel accepts", async () => {
  const { fetchImpl } = registry({
    packageName: "ferromark",
    versionDocument: { name: "ferromark", version: "2.0.0-dev.0" },
    distTags: { next: "2.0.0-dev.0" },
  });

  await assert.rejects(
    verifyNpmPublication({
      packageName: "ferromark",
      version: "2.0.0-dev.0",
      publishResult: "success",
      fetchImpl,
      sleepImpl,
    }),
    /Release version must be stable or an rc/,
  );
});
