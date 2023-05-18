use super::{
    AndroidMainEnum, AndroidMainEnumVariant, AndroidMainStruct, CommonMainEnum, CommonMainStruct,
    KotlinType,
};
use crate::codegen::kotlin::functions::process_android_main_methods;
use crate::codegen::kotlin::inits::process_android_main_inits;
use crate::codegen::kotlin::properties::process_android_main_properties;
use crate::codegen::swift::ObjectVariant;
use crate::manifest::FileInfo;
use crate::Result;
use handlebars::Handlebars;

pub fn pretty_name(name: String) -> String {
    name.replace("_", "").replace("TW", "").replace("Proto", "")
}

/// Convenience wrapper for setting copyright year when generating bindings.
// TODO: Unify with Swift.
#[derive(Debug, Clone, Serialize)]
pub struct WithYear<'a, T> {
    pub current_year: u64,
    #[serde(flatten)]
    pub data: &'a T,
}

pub struct RenderIntput<'a> {
    pub file_info: FileInfo,
    pub android_main_struct: &'a str,
    pub android_main_enum: &'a str,
    pub common_main_struct: &'a str,
    pub common_main_enum: &'a str,
}

// TODO: Rename
#[derive(Debug, Clone, Default)]
pub struct GeneratedPlatformTypes {
    pub android_main: GeneratedTypes<AndroidMainStruct, AndroidMainEnum>,
    pub common_main: GeneratedTypes<CommonMainStruct, CommonMainEnum>,
}

#[derive(Debug, Clone)]
pub struct GeneratedTypes<S, E> {
    pub structs: Vec<S>,
    pub enums: Vec<E>,
}

impl<S, E> Default for GeneratedTypes<S, E> {
    fn default() -> Self {
        GeneratedTypes {
            structs: vec![],
            enums: vec![],
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct GeneratedMultiplatformStrings {
    pub android_main: GeneratedStrings,
    pub common_main: GeneratedStrings,
}

#[derive(Debug, Clone, Default)]
pub struct GeneratedStrings {
    pub structs: Vec<(String, String)>,
    pub enums: Vec<(String, String)>,
}

pub fn render_to_strings<'a>(input: RenderIntput<'a>) -> Result<GeneratedMultiplatformStrings> {
    // The current year for the copyright header in the generated bindings.
    let current_year = crate::current_year();
    // Convert the name into an appropriate format.
    let pretty_file_name = pretty_name(input.file_info.name.clone());

    let mut engine = Handlebars::new();
    engine.set_strict_mode(true);

    engine.register_partial("android_main_struct", input.android_main_struct)?;
    engine.register_partial("android_main_enum", input.android_main_enum)?;
    engine.register_partial("common_main_struct", input.common_main_struct)?;
    engine.register_partial("common_main_enum", input.common_main_enum)?;

    let generated = generate_android_main_types(input.file_info)?;
    let mut out_strings = GeneratedMultiplatformStrings::default();

    for strct in generated.android_main.structs {
        let out = engine.render(
            "android_main_struct",
            &WithYear {
                current_year,
                data: &strct,
            },
        )?;

        out_strings.android_main.structs.push((strct.name, out));
    }

    for enm in generated.android_main.enums {
        let out = engine.render(
            "android_main_enum",
            &WithYear {
                current_year,
                data: &enm,
            },
        )?;

        out_strings.android_main.enums.push((enm.name, out));
    }

    for strct in generated.common_main.structs {
        let out = engine.render(
            "common_main_struct",
            &WithYear {
                current_year,
                data: &strct,
            },
        )?;

        out_strings.common_main.structs.push((strct.name, out));
    }

    Ok(out_strings)
}

pub fn generate_android_main_types(mut info: FileInfo) -> Result<GeneratedPlatformTypes> {
    let mut outputs = GeneratedPlatformTypes::default();

    for strct in info.structs {
        let obj = ObjectVariant::Struct(&strct.name);

        let (inits, methods, properties);
        (inits, info.inits) = process_android_main_inits(&obj, info.inits)?;
        (methods, info.functions) = process_android_main_methods(&obj, info.functions)?;
        (properties, info.properties) = process_android_main_properties(&obj, info.properties)?;

        // Avoid rendering empty structs.
        // TODO: Correct behavior?
        if inits.is_empty() && methods.is_empty() && properties.is_empty() {
            continue;
        }

        // Convert the name into an appropriate format.
        let pretty_name = pretty_name(strct.name.clone());

        outputs.android_main.structs.push(AndroidMainStruct {
            name: pretty_name,
            is_class: strct.is_class,
            inits,
            methods,
            properties,
        });
    }

    for enm in info.enums {
        let obj = ObjectVariant::Enum(&enm.name);

        let (methods, properties);
        (methods, info.functions) = process_android_main_methods(&obj, info.functions)?;
        (properties, info.properties) = process_android_main_properties(&obj, info.properties)?;

        // TODO: Just deprecate `AndroidMainEnumVariant`?
        let variants = enm
            .variants
            .into_iter()
            .map(|enm| AndroidMainEnumVariant {
                name: enm.name,
                value: enm.value,
                as_string: enm.as_string,
            })
            .collect();

        // Convert the name into an appropriate format.
        let pretty_name = pretty_name(enm.name.clone());

        outputs.android_main.enums.push(AndroidMainEnum {
            name: pretty_name,
            value_type: KotlinType::from(enm.value_type),
            methods,
            properties,
            variants,
        })
    }

    Ok(outputs)
}
