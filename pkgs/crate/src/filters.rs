use heck::{
    ToKebabCase, ToLowerCamelCase, ToPascalCase, ToShoutyKebabCase, ToShoutySnakeCase, ToSnakeCase,
    ToTitleCase, ToTrainCase, ToUpperCamelCase,
};
use tera::{Kwargs, State, Tera};

pub(crate) fn register_filters(tera: &mut Tera) {
    macro_rules! register {
        ($($method:ident),+ $(,)?) => {
            $(tera.register_filter(
                stringify!($method),
                |value: &str, _: Kwargs, _: &State| value.$method(),
            );)+
        };
    }

    register!(
        to_upper_camel_case,
        to_pascal_case,
        to_lower_camel_case,
        to_snake_case,
        to_kebab_case,
        to_shouty_snake_case,
        to_shouty_kebab_case,
        to_title_case,
        to_train_case,
    );
}
