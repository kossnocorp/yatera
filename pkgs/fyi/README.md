# FYI.party

A Markdown-driven Astro page.

```astro
---
import Page from "fyi.party/Page.astro";
import markdown from "../../README.md?raw";
---

<Page
  markdown={markdown}
  parseHero
  parseLicense
  parseInstallation
  displayLicenseInFooter
  links={[{ label: "GitHub", href: "https://github.com/owner/repo", icon: "github" }]}
/>
```

`parseHero` defaults to `false`. When enabled, the first top-level H1 and
everything before the next top-level heading form a hero section. Nested
headings in blockquotes or code do not end the hero. Content before the H1 is
preserved, and later H1s remain ordinary document content.

Project links appear immediately below the hero, or above the document when no
hero is detected. The page supports `head`, `favicon`, and `footer` slots.

The hero has separate `header`, `subheadline`, and `description` fields (token
arrays when parsed, HTML strings when rendered). Its first top-level paragraph
becomes the subheadline; all other content becomes the description. Inline
Markdown is preserved. A hero without a paragraph has no subheadline.

The renderer uses two stages internally:

```ts
const parsed = await parseDoc(markdown, { parseHero: true, parseLicense: true });
const doc = await renderDoc(parsed, { displayLicenseInFooter: true });
```

The result includes `title`, `headings`, and ordered `sections`. Each section is
`{ kind: "doc" | "hero" | "license" | "installation", html: string }`. Heading slugs are shared across all
sections, and a parsed document can be rendered repeatedly. `readDoc(path, options)`
combines reading, parsing, and rendering in Node.js.

`parseDoc` and `ParserOptions` live in `src/parser`; rendering and display options
live in `src/renderer`. `parseLicense` recognizes a top-level heading named
"License" (case-insensitive), including inline formatting. Its content extends
through any subsections until the next heading of equal or higher level. The
parsed license section has separate `header` and `content` token arrays.

With `displayLicenseInFooter`, the license header is omitted and its content is
rendered in the footer alongside any `footer` slot content. Otherwise the full
license section stays in its original position. Both license options default to
`false`; footer relocation requires `parseLicense` as well.

`parseInstallation` (default `false`) extracts a heading named "Installation"
(case-insensitive). The section ends at the next heading of equal or higher
level. Its immediate subheadings become tabs; deeper headings stay in their
panel. Text before the first tab stays visible above the tabs. Sections without
subheadings render normally without a tab bar.

Parsed installation sections contain `header`, intro `content`, and `tabs`
with their own `header` and `content` tokens. Rendered sections include `id`,
`label`, intro `html`, and `tabs: { id, label, html }[]`.

The first tab is active initially. Click a tab or use Left/Right, Home, and End
to switch panels. Links to tab headings or headings inside a panel automatically
activate that panel. Only the active panel's content is displayed.
