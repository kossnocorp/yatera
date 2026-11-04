import { describe, expect, it } from "vitest";
import { parseDoc } from "./index.ts";

describe("parseDoc", () => {
  it("splits the first hero paragraph from all remaining description content", async () => {
    const parsed = await parseDoc(
      "# Project\n\n> Quoted intro\n\nA **subtitle**.\n\nDescription.\n\n- Feature\n\n## Usage",
      { parseHero: true },
    );
    const hero = parsed.sections[0];
    if (hero?.kind !== "hero") throw new Error("Missing hero");
    expect(hero.header).toMatchObject([{ type: "heading", depth: 1 }]);
    expect(hero.subheadline).toMatchObject([{ type: "paragraph", text: "A **subtitle**." }]);
    expect(hero.description.filter(({ type }) => type !== "space").map(({ type }) => type)).toEqual(
      ["blockquote", "paragraph", "list"],
    );
    const empty = await parseDoc("# Project\n\n```sh\necho hello\n```", { parseHero: true });
    expect(empty.sections[0]).toMatchObject({ kind: "hero", subheadline: [] });
  });

  it("extracts installation intro and immediate subheadings as tabs", async () => {
    const markdown =
      "# Project\n\n## **Installation**\n\nChoose a method.\n\n### macOS\n\n```sh\n# Not a heading\nbrew install project\n```\n\n#### Troubleshooting\n\nHelp.\n\n### Windows\n\nInstall.\n\n## Usage\n\nRun.";
    expect((await parseDoc(markdown)).sections.map(({ kind }) => kind)).toEqual(["doc"]);
    const parsed = await parseDoc(markdown, { parseHero: true, parseInstallation: true });
    expect(parsed.sections.map(({ kind }) => kind)).toEqual(["hero", "installation", "doc"]);
    const section = parsed.sections[1];
    if (section?.kind !== "installation") throw new Error("Missing installation");
    expect(section.header).toMatchObject([{ type: "heading", depth: 2 }]);
    expect(section.content).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ type: "paragraph", text: "Choose a method." }),
      ]),
    );
    expect(
      section.tabs.map(({ header }) => (header[0]?.type === "heading" ? header[0].text : "")),
    ).toEqual(["macOS", "Windows"]);
    expect(section.tabs[0]?.content).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ type: "heading", depth: 4, text: "Troubleshooting" }),
      ]),
    );
    expect(
      section.tabs[1]?.content.some((token) => token.type === "heading" && token.text === "Usage"),
    ).toBe(false);
  });

  it("preserves installation sections without tabs and ignores false matches", async () => {
    const parsed = await parseDoc("## INSTALLATION\n\nJust install it.", {
      parseInstallation: true,
    });
    expect(parsed.sections).toMatchObject([{ kind: "installation", tabs: [] }]);
    for (const markdown of [
      "> ## Installation",
      "```md\n## Installation\n```",
      "## Installation notes",
    ]) {
      expect(
        (await parseDoc(markdown, { parseInstallation: true })).sections.map(({ kind }) => kind),
      ).toEqual(["doc"]);
    }
  });

  it("separates license headers and content while retaining subsections and following content", async () => {
    const markdown =
      "# Project\n\nIntro\n\n## **LICENSE**\n\n[MIT][license]\n\n### Attribution\n\nAuthor\n\n## More\n\nEnd\n\n[license]: https://example.com/license";
    const parsed = await parseDoc(markdown, { parseHero: true, parseLicense: true });
    expect(parsed.sections.map(({ kind }) => kind)).toEqual(["hero", "license", "doc"]);
    const license = parsed.sections[1];
    expect(license?.kind).toBe("license");
    if (license?.kind !== "license") throw new Error("Missing license");
    expect(license.header).toMatchObject([{ type: "heading", depth: 2 }]);
    expect(
      license.content.some((token) => token.type === "heading" && token.text === "Attribution"),
    ).toBe(true);
    expect(license.content.some((token) => token.type === "heading" && token.text === "More")).toBe(
      false,
    );
  });

  it("keeps parsing opt-in and ignores quoted headings, code, and partial matches", async () => {
    expect((await parseDoc("## License\n\nMIT")).sections.map(({ kind }) => kind)).toEqual(["doc"]);
    for (const markdown of [
      "",
      "> # License",
      "```md\n# License\n```",
      "## License details\n\nText",
    ]) {
      expect(
        (await parseDoc(markdown, { parseLicense: true })).sections.map(({ kind }) => kind),
      ).toEqual(["doc"]);
    }
  });

  it("prioritizes a license H1 over hero detection and supports an empty license", async () => {
    const parsed = await parseDoc("# License", { parseHero: true, parseLicense: true });
    expect(parsed.sections).toMatchObject([
      { kind: "license", header: [{ type: "heading" }], content: [] },
    ]);
  });
});
