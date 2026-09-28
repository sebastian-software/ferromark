# HTTP API integration for the README

The API gateway accepts JSON over HTTPS and supports HTTP/2. Use the SDK to
create a GraphQL request, then inspect its HTTP response and URL. The README
MUST include the API endpoint. A clientID is different from an ID, and
API_TOKEN belongs in code examples rather than prose.

![API flow](/images/api-flow.svg)

```js
const API_TOKEN = process.env.API_TOKEN;
```
