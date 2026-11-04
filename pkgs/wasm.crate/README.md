# Yatera WebAssembly

`yatera_wasm` wraps the Yatera renderer for browsers. Build the npm package into
`pkgs/wasm.npm` from the repository root:

```sh
mise run //pkgs/wasm.crate:build
```

The task uses wasm-pack's web target and jaq to set the generated npm package name
to `@yatera.fyi/wasm`. Generated JavaScript, TypeScript declarations, and Wasm are
ignored by Git. The package manifest is tracked for pnpm workspace discovery.

```js
import init, { render } from "@yatera.fyi/wasm";

await init();
const output = render("{{ name | snake_case }}", '{"name":"Hello World"}');
// hello_world
```

`render(source, input)` takes a Tera template and a JSON object encoded as a string.
It returns plain text and throws on invalid input or template errors. It includes
the same filters and disables HTML autoescaping just like the native renderer.

Use `render_with_plugins(source, input, plugins)` to pass Lua sources as a string
array. The Lua API is shared with the native crate (see `pkgs/crate/README.md`):

```js
import init, { render_with_plugins } from "@yatera.fyi/wasm";

await init();
const output = render_with_plugins("{{ name | shout }}", '{"name":"World"}', [
  'yatera.filters.shout = function(value, args) return string.upper(value) .. "!" end',
]);
// WORLD!
```

For a development build, run `mise run //pkgs/wasm.crate:build:dev` (or pass
`--dev` to `build`). The website's `dev` task uses this development build;
its `build` and `deploy` tasks use the default release build.
