import { readFile } from "node:fs/promises";
import { parseDoc, textContent, type ParsedDocument, type ParserOptions } from "../parser/index.ts";
import GithubSlugger from "github-slugger";
import { Marked, type RendererObject, type Token } from "marked";
import {
  createHighlighter,
  bundledLanguages,
  createCssVariablesTheme,
  type BundledLanguage,
} from "shiki";
import { createJavaScriptRegexEngine } from "shiki/engine/javascript";

const theme = createCssVariablesTheme({ variablePrefix: "--astro-code-" });
let highlighter: ReturnType<typeof createHighlighter> | undefined;

export interface Document {
  sections: PageSection[];
  title?: string;
  headings: { depth: number; slug: string; text: string }[];
}

export interface DisplayOptions {
  displayLicenseInFooter?: boolean;
}

export interface RendererOptions extends DisplayOptions {}

export interface PageSectionBase<Kind extends string> {
  kind: Kind;
  html: string;
}

export interface PageSectionDoc extends PageSectionBase<"doc"> {}
export interface PageSectionHero extends PageSectionBase<"hero"> {
  header: string;
  subheadline: string;
  description: string;
}
export interface PageSectionLicense extends PageSectionBase<"license"> {}
export interface PageSectionInstallation extends PageSectionBase<"installation"> {
  id: string;
  label: string;
  tabs: { id: string; label: string; html: string }[];
}
export type PageSection =
  | PageSectionDoc
  | PageSectionHero
  | PageSectionLicense
  | PageSectionInstallation;

/**
 * Render repository Markdown with heading anchors, GFM, and highlighted code.
 */
export async function renderDoc(
  parsed: ParsedDocument,
  options: RendererOptions = {},
): Promise<Document> {
  const headings: Document["headings"] = [];
  const slugger = new GithubSlugger();

  const renderer: RendererObject = {
    heading({ tokens, depth }) {
      const text = textContent(tokens);
      const slug = slugger.slug(text);
      headings.push({ depth, slug, text });
      const content = `<h${depth} id="${slug}">${this.parser.parseInline(tokens)}</h${depth}>\n`;
      if (depth === 1) return content + "\n";
      return `<div class="fyi-heading">${content}\n<a href="#${slug}" class="fyi-anchor">#</a></div>\n`;
    },

    code({ text }) {
      return text;
    },
  };

  const processor = new Marked({ gfm: true, renderer });
  async function renderTokens(source: Token[]): Promise<string> {
    // Highlight a copy so the parsed document can be rendered more than once.
    const tokens = structuredClone(source);
    await Promise.all(
      processor.walkTokens(tokens, async (token) => {
        if (token.type === "code") {
          token.text = await highlight(token.text, token.lang?.split(/\s+/)[0] ?? "text");
        }
      }),
    );
    return processor.parser(tokens);
  }

  const sections: PageSection[] = [];
  for (const section of parsed.sections) {
    if (section.kind === "hero") {
      const header = await renderTokens(section.header);
      const subheadline = await renderTokens(section.subheadline);
      const description = await renderTokens(section.description);
      sections.push({
        kind: "hero",
        header,
        subheadline,
        description,
        html: header + subheadline + description,
      });
    } else if (section.kind === "installation") {
      const headerIndex = headings.length;
      const header = await renderTokens(section.header);
      const heading = headings[headerIndex]!;
      const html = header + (await renderTokens(section.content));
      const tabs: PageSectionInstallation["tabs"] = [];
      for (const tab of section.tabs) {
        // Reserve the original heading slug in document order for tab links.
        const tabIndex = headings.length;
        await renderTokens(tab.header);
        const tabHeading = headings[tabIndex]!;
        tabs.push({
          id: tabHeading.slug,
          label: tabHeading.text,
          html: await renderTokens(tab.content),
        });
      }
      sections.push({ kind: "installation", id: heading.slug, label: heading.text, html, tabs });
    } else {
      const tokens =
        section.kind === "license"
          ? [...(options.displayLicenseInFooter ? [] : section.header), ...section.content]
          : section.tokens;
      sections.push({ kind: section.kind, html: await renderTokens(tokens) });
    }
  }

  const title = headings.find((heading) => heading.depth === 1)?.text;

  return {
    sections,
    title,
    headings,
  };
}

async function highlight(code: string, language: string): Promise<string> {
  const lang = Object.hasOwn(bundledLanguages, language) ? (language as BundledLanguage) : "text";
  highlighter ??= createHighlighter({
    themes: [theme],
    langs: [],
    engine: createJavaScriptRegexEngine(),
  });

  const renderer = await highlighter;
  if (lang !== "text") await renderer.loadLanguage(lang);

  return renderer.codeToHtml(code, {
    lang,
    theme,
  });
}

/**
 * Read a local README at build time.
 */
export async function readDoc(
  path: string | URL,
  options?: ParserOptions & RendererOptions,
): Promise<Document> {
  return renderDoc(await parseDoc(await readFile(path, "utf8"), options), options);
}
