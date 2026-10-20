export function ApiNotebook() {
  return (
    <dl className="ferromark-notebook">
      <div>
        <dt>Render one document</dt>
        <dd>
          <code>to_html()</code>
        </dd>
      </div>
      <div>
        <dt>Choose output policy</dt>
        <dd>
          <code>to_html_with_options()</code>
        </dd>
      </div>
      <div>
        <dt>Inspect structure</dt>
        <dd>Allocator + Parser + HtmlRenderer</dd>
      </div>
      <div>
        <dt>Customize output</dt>
        <dd>
          <code>HtmlRenderHooks</code>
        </dd>
      </div>
    </dl>
  );
}
