import { readFile } from "node:fs/promises";
import GithubSlugger from "github-slugger";
import { Marked, type RendererObject, type Token } from "marked";
import { markedHighlight } from "marked-highlight";
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
  html: string;
  title?: string;
  headings: { depth: number; slug: string; text: string }[];
}

function textContent(tokens: Token[]): string {
  return tokens
    .map((token) => {
      if ("tokens" in token && token.tokens) return textContent(token.tokens);
      if (token.type === "html") return "";
      return "text" in token ? token.text : "";
    })
    .join("");
}

/**
 * Render repository Markdown with heading anchors, GFM, and highlighted code.
 */
export async function renderDoc(markdown: string): Promise<Document> {
  const headings: Document["headings"] = [];
  const slugger = new GithubSlugger();

  const markedExtension = markedHighlight({ async: true, highlight });

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

  const processor = new Marked(markedExtension, { gfm: true, renderer });
  const html = await processor.parse(markdown, { async: true });

  const title = headings.find((heading) => heading.depth === 1)?.text;

  return {
    html,
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
export async function readDoc(path: string | URL): Promise<Document> {
  return renderDoc(await readFile(path, "utf8"));
}
