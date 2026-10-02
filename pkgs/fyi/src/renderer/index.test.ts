import { describe, it, expect } from "vitest";
import { readDoc, renderDoc } from "./index.ts";

describe("renderDoc", () => {
  it("renders GFM markdown", async () => {
    const doc = await renderDoc(
      [
        "# My **project**",
        "## Usage",
        "## Usage",
        "| Option | Value |\n| --- | --- |\n| `--flag` | yes |",
        "```bash\necho '<hello>'\n```",
        "```unknown-language\n<raw> & text\n```",
      ].join("\n\n"),
    );

    expect(doc.title).toBe("My project");

    expect(doc.headings).toEqual([
      { depth: 1, slug: "my-project", text: "My project" },
      { depth: 2, slug: "usage", text: "Usage" },
      { depth: 2, slug: "usage-1", text: "Usage" },
    ]);

    expect(doc.html).toMatchInlineSnapshot(`
      "<h1 id="my-project">My <strong>project</strong></h1>

      <div class="fyi-heading"><h2 id="usage">Usage</h2>

      <a href="#usage" class="fyi-anchor">#</a></div>
      <div class="fyi-heading"><h2 id="usage-1">Usage</h2>

      <a href="#usage-1" class="fyi-anchor">#</a></div>
      <table>
      <thead>
      <tr>
      <th>Option</th>
      <th>Value</th>
      </tr>
      </thead>
      <tbody><tr>
      <td><code>--flag</code></td>
      <td>yes</td>
      </tr>
      </tbody></table>
      <pre class="shiki css-variables" style="background-color:var(--astro-code-background);color:var(--astro-code-foreground)" tabindex="0"><code><span class="line"><span style="color:var(--astro-code-token-function)">echo</span><span style="color:var(--astro-code-token-string-expression)"> '&#x3C;hello>'</span></span></code></pre><pre class="shiki css-variables" style="background-color:var(--astro-code-background);color:var(--astro-code-foreground)" tabindex="0"><code><span class="line"><span>&#x3C;raw> &#x26; text</span></span></code></pre>"
    `);
  });
});
