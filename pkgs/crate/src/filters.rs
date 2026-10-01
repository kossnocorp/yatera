use heck::{
    ToKebabCase, ToLowerCamelCase, ToPascalCase, ToShoutyKebabCase, ToShoutySnakeCase, ToSnakeCase,
    ToTrainCase, ToUpperCamelCase,
};
use tera::{Kwargs, State, Tera};

pub(crate) fn register_filters(tera: &mut Tera) {
    macro_rules! register {
        ($($method:ident => $name:ident),+ $(,)?) => {
            $(tera.register_filter(
                stringify!($name),
                |value: &str, _: Kwargs, _: &State| value.$method(),
            );)+
        };
    }

    register!(
        to_upper_camel_case => upper_camel_case,
        to_pascal_case => pascal_case,
        to_lower_camel_case => lower_camel_case,
        to_snake_case => snake_case,
        to_kebab_case => kebab_case,
        to_shouty_snake_case => shouty_snake_case,
        to_shouty_kebab_case => shouty_kebab_case,
        to_train_case => train_case,
    );
}
