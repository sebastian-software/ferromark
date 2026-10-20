import type { EntryContext } from "ardo/router";

import { renderToReadableStream } from "ardo/react-dom/server";
import { ServerRouter } from "ardo/router";
import { isbot } from "isbot";

/* eslint-disable max-params -- React Router requires this four-argument entry point. */
// oxlint-disable-next-line max-params -- React Router requires this four-argument entry point.
export default async function handleRequest(
  request: Request,
  responseStatusCode: number,
  responseHeaders: Headers,
  routerContext: EntryContext,
) {
  const userAgent = request.headers.get("user-agent");

  const stream = await renderToReadableStream(
    <ServerRouter context={routerContext} url={request.url} />,
    {
      onError(error: unknown) {
        console.error(error);
        responseStatusCode = 500;
      },
    },
  );

  if (userAgent !== null && userAgent !== "" && isbot(userAgent)) {
    await stream.allReady;
  }

  responseHeaders.set("Content-Type", "text/html");

  return new Response(stream, {
    status: responseStatusCode,
    headers: responseHeaders,
  });
}
