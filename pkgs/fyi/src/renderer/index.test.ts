import { describe, it, expect } from "vitest";
import { parseDoc } from "../parser/index.ts";
import { renderDoc } from "./index.ts";

describe("renderDoc", () => {
  it("renders hero fields separately with inline Markdown intact", async () => {
    const parsed = await parseDoc(
      "# Project\n\nA **great** [tool][tool].\n\nLonger description.\n\n```sh\necho hello\n```\n\n[tool]: https://example.com",
      { parseHero: true },
    );
    const doc = await renderDoc(parsed);
    const hero = doc.sections[0];
    if (hero?.kind !== "hero") throw new Error("Missing hero");
    expect(hero.header).toContain('<h1 id="project">');
    expect(hero.subheadline).toBe(
      '<p>A <strong>great</strong> <a href="https://example.com">tool</a>.</p>\n',
    );
    expect(hero.description).toContain("Longer description.");
    expect(hero.description).toContain('class="shiki');
    expect(hero.description).not.toContain("<strong>great</strong>");
    expect(await renderDoc(parsed)).toEqual(doc);
    const titleOnly = await renderDoc(await parseDoc("# Project", { parseHero: true }));
    expect(titleOnly.sections[0]).toMatchObject({ subheadline: "", description: "" });
  });

  it("renders installation tabs with shared heading slugs and highlighted content", async () => {
    const parsed = await parseDoc(
      "# Project\n\n## Installation\n\nPick one.\n\n### **Shell**\n\n```sh\necho install\n```\n\n#### Details\n\n[More][more]\n\n### Shell\n\nAlternative\n\n## Shell\n\nAfter\n\n[more]: https://example.com",
      { parseInstallation: true },
    );
    const doc = await renderDoc(parsed);
    const section = doc.sections[1];
    if (section?.kind !== "installation") throw new Error("Missing installation");
    expect(section.id).toBe("installation");
    expect(section.label).toBe("Installation");
    expect(section.html).toContain("Pick one.");
    expect(section.html).not.toContain("echo install");
    expect(section.tabs.map(({ id, label }) => ({ id, label }))).toEqual([
      { id: "shell", label: "Shell" },
      { id: "shell-1", label: "Shell" },
    ]);
    expect(section.tabs[0]?.html).toContain('class="shiki');
    expect(section.tabs[0]?.html).toContain('id="details"');
    expect(section.tabs[0]?.html).toContain('href="https://example.com"');
    expect(section.tabs[0]?.html).not.toContain("<h3");
    expect(doc.sections[2]?.html).toContain('id="shell-2"');
    expect(await renderDoc(parsed)).toEqual(doc);
  });

  it("renders license content without its header only in footer mode", async () => {
    const parsed = await parseDoc(
      "# Project\n\n## License\n\n[MIT][mit]\n\n[mit]: https://example.com/mit",
      { parseLicense: true },
    );
    const inline = await renderDoc(parsed);
    expect(inline.sections[1]?.html).toContain('<h2 id="license">');
    const footer = await renderDoc(parsed, { displayLicenseInFooter: true });
    expect(footer.sections[1]).toEqual({
      kind: "license",
      html: '<p><a href="https://example.com/mit">MIT</a></p>\n',
    });
    expect(footer.headings.map(({ slug }) => slug)).toEqual(["project"]);
    expect(await renderDoc(parsed)).toEqual(inline);
    const plain = await renderDoc(await parseDoc("## License\n\nMIT"), {
      displayLicenseInFooter: true,
    });
    expect(plain.sections[0]?.kind).toBe("doc");
    expect(plain.sections[0]?.html).toContain('<h2 id="license">');
  });

  it("detects an opt-in hero through the next top-level heading", async () => {
    const markdown =
      "# Intro\n\nWelcome [here][docs].\n\n> ## Nested heading\n\n```md\n# Code heading\n```\n\n## Intro\n\nBody.\n\n[docs]: https://example.com";
    const plain = await renderDoc(await parseDoc(markdown));
    expect(plain.sections.map(({ kind }) => kind)).toEqual(["doc"]);

    const parsed = await parseDoc(markdown, { parseHero: true });
    const doc = await renderDoc(parsed);
    expect(doc.sections.map(({ kind }) => kind)).toEqual(["hero", "doc"]);
    expect(doc.sections[0]?.html).toContain('<h1 id="intro">Intro</h1>');
    expect(doc.sections[0]?.html).toContain('href="https://example.com"');
    expect(doc.sections[0]?.html).toContain("Nested heading");
    expect(doc.sections[0]?.html).toContain("Code heading");
    expect(doc.sections[0]?.html).not.toContain("Body.");
    expect(doc.sections[1]?.html).toContain('<h2 id="intro-1">');
    expect(doc.sections[1]?.html).toContain("Body.");
    expect(await renderDoc(parsed)).toEqual(doc);
  });

  it("keeps content before the hero and detects only one hero", async () => {
    const doc = await renderDoc(
      await parseDoc("Preface.\n\n# First\n\nIntro.\n\n# Second\n\nMore.", { parseHero: true }),
    );
    expect(doc.sections.map(({ kind }) => kind)).toEqual(["doc", "hero", "doc"]);
    expect(doc.sections[0]?.html).toContain("Preface.");
    expect(doc.sections[1]?.html).toContain("Intro.");
    expect(doc.sections[2]?.html).toContain("Second");
  });

  it("handles hero-only documents and documents without a top-level H1", async () => {
    const hero = await renderDoc(
      await parseDoc("# Header\n\nSome intro text", { parseHero: true }),
    );
    expect(hero.sections.map(({ kind }) => kind)).toEqual(["hero"]);
    expect(hero.sections[0]?.html).toContain("Some intro text");
    for (const markdown of ["", "## Usage\n\nBody", "> # Quoted title"]) {
      const doc = await renderDoc(await parseDoc(markdown, { parseHero: true }));
      expect(doc.sections.map(({ kind }) => kind)).toEqual(["doc"]);
    }
  });

  it("renders GFM markdown", async () => {
    const doc = await renderDoc(
      await parseDoc(
        [
          "# My **project**",
          "## Usage",
          "## Usage",
          "| Option | Value |\n| --- | --- |\n| `--flag` | yes |",
          "```bash\necho '<hello>'\n```",
          "```unknown-language\n<raw> & text\n```",
        ].join("\n\n"),
      ),
    );

    expect(doc.title).toBe("My project");

    expect(doc.headings).toEqual([
      { depth: 1, slug: "my-project", text: "My project" },
      { depth: 2, slug: "usage", text: "Usage" },
      { depth: 2, slug: "usage-1", text: "Usage" },
    ]);

    expect(doc.sections[0]?.html).toMatchInlineSnapshot(`
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
