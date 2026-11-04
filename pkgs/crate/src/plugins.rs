use std::cell::RefCell;

use piccolo::{Closure, Executor, Fuel, Lua, StashedExecutor, Table, Value as LuaValue};
use serde_json::Value as Json;
use tera::{Context, Error, Kwargs, State, Tera, Value};

// Piccolo's arena is thread-bound. Tera callbacks require Send + Sync, so they
// capture only a slot ID. The private Tera instance and runtime never leave the
// synchronous render call; the guard removes the arena even on errors/unwinds.
thread_local! {
    static RUNTIMES: RefCell<Vec<Option<Lua>>> = const { RefCell::new(Vec::new()) };
}

pub(crate) struct Runtime(usize);

impl Drop for Runtime {
    fn drop(&mut self) {
        RUNTIMES.with_borrow_mut(|runtimes| runtimes[self.0] = None);
    }
}

fn error(error: impl std::fmt::Display) -> Error {
    Error::message(format!("Lua plugin: {error}"))
}

// Yield periodically for GC, and turn runaway plugin code into a render error.
fn finish(lua: &mut Lua, executor: &StashedExecutor) -> Result<(), Error> {
    for _ in 0..256 {
        let mut fuel = Fuel::with(4096);
        if lua.enter(|ctx| ctx.fetch(executor).step(ctx, &mut fuel)) {
            return Ok(());
        }
    }
    Err(error("execution limit exceeded"))
}

fn table<'gc>(value: LuaValue<'gc>) -> Result<Table<'gc>, Error> {
    match value {
        LuaValue::Table(table) => Ok(table),
        _ => Err(error("expected a table")),
    }
}

fn to_lua<'gc>(ctx: piccolo::Context<'gc>, value: &Json) -> Result<LuaValue<'gc>, Error> {
    Ok(match value {
        Json::Null => table(ctx.get_global("yatera"))?.get(ctx, "null"),
        Json::Bool(value) => LuaValue::Boolean(*value),
        Json::Number(value) => match value.as_i64() {
            Some(value) => LuaValue::Integer(value),
            None if value.is_u64() => {
                return Err(error("integer exceeds Lua's signed 64-bit range"));
            }
            None => LuaValue::Number(value.as_f64().ok_or_else(|| error("invalid number"))?),
        },
        Json::String(value) => LuaValue::String(ctx.intern(value.as_bytes())),
        Json::Array(values) => {
            let result = Table::new(&ctx);
            let meta = Table::new(&ctx);
            meta.set(ctx, "__yatera_array", true).map_err(error)?;
            result.set_metatable(&ctx, Some(meta));
            for (index, value) in values.iter().enumerate() {
                result
                    .set(ctx, index as i64 + 1, to_lua(ctx, value)?)
                    .map_err(error)?;
            }
            LuaValue::Table(result)
        }
        Json::Object(values) => {
            let result = Table::new(&ctx);
            for (key, value) in values {
                result
                    .set(ctx, ctx.intern(key.as_bytes()), to_lua(ctx, value)?)
                    .map_err(error)?;
            }
            LuaValue::Table(result)
        }
    })
}

fn from_lua<'gc>(
    ctx: piccolo::Context<'gc>,
    value: LuaValue<'gc>,
    depth: usize,
) -> Result<Json, Error> {
    if depth > 64 {
        return Err(error(
            "returned table is cyclic or nested more than 64 levels",
        ));
    }
    Ok(match value {
        LuaValue::Nil => Json::Null,
        LuaValue::Boolean(value) => Json::Bool(value),
        LuaValue::Integer(value) => value.into(),
        LuaValue::Number(value) => serde_json::Number::from_f64(value)
            .ok_or_else(|| error("cannot return a non-finite number"))?
            .into(),
        LuaValue::String(value) => std::str::from_utf8(value.as_bytes()).map_err(error)?.into(),
        LuaValue::Table(value) => {
            if value == table(table(ctx.get_global("yatera"))?.get(ctx, "null"))? {
                return Ok(Json::Null);
            }
            let entries: Vec<_> = value.iter().collect();
            let array = value
                .metatable()
                .is_some_and(|meta| meta.get(ctx, "__yatera_array").to_bool())
                || (!entries.is_empty()
                    && entries
                        .iter()
                        .all(|(key, _)| matches!(key, LuaValue::Integer(_))));
            if array {
                let len = entries.len();
                if !entries.iter().all(
                    |(key, _)| matches!(key, LuaValue::Integer(i) if *i > 0 && *i <= len as i64),
                ) {
                    return Err(error(
                        "arrays must have consecutive integer keys starting at 1",
                    ));
                }
                Json::Array(
                    (1..=len)
                        .map(|index| from_lua(ctx, value.get(ctx, index as i64), depth + 1))
                        .collect::<Result<_, _>>()?,
                )
            } else {
                let mut result = serde_json::Map::new();
                for (key, value) in entries {
                    let LuaValue::String(key) = key else {
                        return Err(error("objects must have string keys"));
                    };
                    result.insert(
                        std::str::from_utf8(key.as_bytes())
                            .map_err(error)?
                            .to_owned(),
                        from_lua(ctx, value, depth + 1)?,
                    );
                }
                Json::Object(result)
            }
        }
        _ => {
            return Err(error(
                "return a scalar, array, or object, not a function/thread/userdata",
            ));
        }
    })
}

pub(crate) fn register(
    tera: &mut Tera,
    context: &mut Context,
    sources: &[String],
) -> Result<Option<Runtime>, Error> {
    if sources.is_empty() {
        return Ok(None);
    }
    let mut lua = Lua::core();
    lua.enter(|ctx| -> Result<(), Error> {
        let api = Table::new(&ctx);
        for name in ["filters", "functions", "tests", "null", "context"] {
            api.set(ctx, name, Table::new(&ctx)).map_err(error)?;
        }
        let context = context.clone();
        api.set(
            ctx,
            "get",
            piccolo::Callback::from_fn(&ctx, move |ctx, _, mut stack| {
                let name: String = stack.consume(ctx)?;
                let overrides = table(ctx.get_global("yatera"))
                    .and_then(|api| table(api.get(ctx, "context")))
                    .map_err(|err| piccolo::Error::from(anyhow::anyhow!(err.to_string())))?;
                let value = overrides.get(ctx, ctx.intern(name.as_bytes()));
                if !value.is_nil() {
                    stack.replace(ctx, value);
                    return Ok(piccolo::CallbackReturn::Return);
                }
                let value = context
                    .get(&name)
                    .map(serde_json::to_value)
                    .transpose()
                    .map_err(|err| piccolo::Error::from(anyhow::anyhow!(err.to_string())))?;
                let value = match value {
                    Some(value) => to_lua(ctx, &value)
                        .map_err(|err| piccolo::Error::from(anyhow::anyhow!(err.to_string())))?,
                    None => LuaValue::Nil,
                };
                stack.replace(ctx, value);
                Ok(piccolo::CallbackReturn::Return)
            }),
        )
        .map_err(error)?;
        ctx.set_global("yatera", api).map_err(error)?;
        Ok(())
    })?;
    for (index, source) in sources.iter().enumerate() {
        let result = (|| {
            let executor = lua
                .try_enter(|ctx| {
                    let closure = Closure::load(
                        ctx,
                        Some(&format!("plugin {}", index + 1)),
                        source.as_bytes(),
                    )?;
                    Ok(ctx.stash(Executor::start(ctx, closure.into(), ())))
                })
                .map_err(error)?;
            finish(&mut lua, &executor)?;
            lua.try_enter(|ctx| ctx.fetch(&executor).take_result::<()>(ctx)?)
                .map_err(error)
        })();
        result.map_err(|err| error(format!("source {}: {err}", index + 1)))?;
    }
    let exports = lua.enter(|ctx| -> Result<Vec<(&'static str, String)>, Error> {
        let api = table(ctx.get_global("yatera"))?;
        let mut exports = Vec::new();
        for kind in ["filters", "functions", "tests"] {
            for (name, function) in table(api.get(ctx, kind))?.iter() {
                let LuaValue::String(name) = name else {
                    return Err(error("export names must be strings"));
                };
                let name = std::str::from_utf8(name.as_bytes())
                    .map_err(error)?
                    .to_owned();
                if !matches!(function, LuaValue::Function(_)) {
                    return Err(error(format!("{kind}.{name} must be a function")));
                }
                exports.push((kind, name));
            }
        }
        Ok(exports)
    })?;
    let additions = lua
        .enter(|ctx| {
            let api = table(ctx.get_global("yatera"))?;
            let additions = from_lua(ctx, LuaValue::Table(table(api.get(ctx, "context"))?), 0)?;
            Context::from_serialize(&additions)
        })
        .map_err(|err| error(format!("context: {err}")))?;
    context.extend(additions);
    let slot = RUNTIMES.with_borrow_mut(|runtimes| {
        let slot = runtimes
            .iter()
            .position(Option::is_none)
            .unwrap_or(runtimes.len());
        if slot == runtimes.len() {
            runtimes.push(None);
        }
        runtimes[slot] = Some(lua);
        slot
    });
    let guard = Runtime(slot);
    for (kind, name) in exports {
        let export = name.clone();
        match kind {
            "filters" => {
                tera.register_filter(name, move |value: &Value, kwargs: Kwargs, _: &State| {
                    call(slot, kind, &export, Some(value), kwargs)
                })
            }
            "functions" => tera.register_function(name, move |kwargs: Kwargs, _: &State| {
                call(slot, kind, &export, None, kwargs)
            }),
            "tests" => tera.register_test(name, move |value: &Value, kwargs: Kwargs, _: &State| {
                bool::try_from(call(slot, kind, &export, Some(value), kwargs)?)
            }),
            _ => unreachable!(),
        }
    }
    Ok(Some(guard))
}

fn call(
    slot: usize,
    kind: &str,
    name: &str,
    value: Option<&Value>,
    kwargs: Kwargs,
) -> Result<Value, Error> {
    let args: serde_json::Map<String, Json> = kwargs
        .iter()
        .map(|(key, value)| Ok((key.to_string(), serde_json::to_value(value).map_err(error)?)))
        .collect::<Result<_, Error>>()?;
    let value = value.map(serde_json::to_value).transpose().map_err(error)?;
    RUNTIMES
        .with_borrow_mut(|runtimes| {
            let lua = runtimes[slot]
                .as_mut()
                .expect("runtime lives throughout rendering");
            let executor = lua.enter(|ctx| -> Result<_, Error> {
                let function =
                    table(table(ctx.get_global("yatera"))?.get(ctx, ctx.intern(kind.as_bytes())))?
                        .get(ctx, ctx.intern(name.as_bytes()));
                let LuaValue::Function(function) = function else {
                    return Err(error(format!("{kind}.{name} is no longer a function")));
                };
                let args = to_lua(ctx, &Json::Object(args))?;
                let values = match value {
                    Some(value) => vec![to_lua(ctx, &value)?, args],
                    None => vec![args],
                };
                Ok(ctx.stash(Executor::start(ctx, function, piccolo::Variadic(values))))
            })?;
            finish(lua, &executor)?;
            lua.enter(|ctx| {
                let value = ctx
                    .fetch(&executor)
                    .take_result::<LuaValue>(ctx)
                    .map_err(error)?
                    .map_err(error)?;
                Value::try_from_serializable(&from_lua(ctx, value, 0)?)
            })
        })
        .map_err(|err| error(format!("{kind}.{name}: {err}")))
}
