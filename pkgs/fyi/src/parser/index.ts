import { Marked, type Token } from "marked";

export interface ParserOptions {
  parseHero?: boolean;
  parseLicense?: boolean;
  parseInstallation?: boolean;
}

export interface PageSectionLicense {
  kind: "license";
  header: Token[];
  content: Token[];
}

export interface PageSectionHero {
  kind: "hero";
  header: Token[];
  subheadline: Token[];
  description: Token[];
}

export interface ParsedDocument {
  sections: (
    | { kind: "doc"; tokens: Token[] }
    | PageSectionHero
    | PageSectionLicense
    | PageSectionInstallation
  )[];
}

export interface PageSectionInstallation {
  kind: "installation";
  header: Token[];
  content: Token[];
  tabs: { header: Token[]; content: Token[] }[];
}

function parseInstallationSection(
  header: Token,
  tokens: Token[],
  depth: number,
): PageSectionInstallation {
  const section: PageSectionInstallation = {
    kind: "installation",
    header: [header],
    content: [],
    tabs: [],
  };
  let content = section.content;
  for (const token of tokens) {
    if (token.type === "heading" && token.depth === depth + 1) {
      const tab = { header: [token], content: [] as Token[] };
      section.tabs.push(tab);
      content = tab.content;
    } else {
      content.push(token);
    }
  }
  return section;
}

export function textContent(tokens: Token[]): string {
  return tokens
    .map((token) => {
      if ("tokens" in token && token.tokens) return textContent(token.tokens);
      if (token.type === "html") return "";
      return "text" in token ? token.text : "";
    })
    .join("");
}

/** Identify special sections among top-level Markdown tokens. */
export async function parseDoc(
  markdown: string,
  options: ParserOptions = {},
): Promise<ParsedDocument> {
  const tokens = new Marked({ gfm: true }).lexer(markdown);
  const sections: ParsedDocument["sections"] = [];
  let start = 0;
  let hasHero = false;

  for (let index = 0; index < tokens.length; index++) {
    const token = tokens[index]!;
    if (token.type !== "heading") continue;
    const license =
      options.parseLicense &&
      textContent(token.tokens ?? [])
        .trim()
        .toLowerCase() === "license";
    const installation =
      options.parseInstallation &&
      textContent(token.tokens ?? [])
        .trim()
        .toLowerCase() === "installation";
    const hero = !license && !installation && options.parseHero && !hasHero && token.depth === 1;
    if (!license && !installation && !hero) continue;

    if (index > start) sections.push({ kind: "doc", tokens: tokens.slice(start, index) });
    let end = index + 1;
    while (end < tokens.length) {
      const next = tokens[end]!;
      // Special sections include subsections; a hero ends at any heading.
      if (next.type === "heading" && (hero || next.depth <= token.depth)) break;
      end++;
    }
    if (license) {
      sections.push({ kind: "license", header: [token], content: tokens.slice(index + 1, end) });
    } else if (installation) {
      sections.push(parseInstallationSection(token, tokens.slice(index + 1, end), token.depth));
    } else {
      const content = tokens.slice(index + 1, end);
      const paragraph = content.findIndex((token) => token.type === "paragraph");
      sections.push({
        kind: "hero",
        header: [token],
        subheadline: paragraph === -1 ? [] : [content[paragraph]!],
        description: content.filter((_, index) => index !== paragraph),
      });
      hasHero = true;
    }
    start = end;
    index = end - 1;
  }

  if (start < tokens.length || sections.length === 0) {
    sections.push({ kind: "doc", tokens: tokens.slice(start) });
  }
  return { sections };
}
