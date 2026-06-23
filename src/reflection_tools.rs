//! Code that makes working with Bevy's reflection system easier.

use bevy::{
    prelude::*,
    reflect::{
        ReflectCloneError, ReflectRef, TypeInfo,
        array::Array,
        enums::{Enum, VariantType},
        list::List,
        map::Map,
        set::Set,
        tuple::Tuple,
    },
};

/// Clones a reflected value, recovering from errors where possible to produce a partially usable clone.
///
/// This is useful for working with reflected values that may contain non-cloneable fields.
/// The result may be incomplete: `#[reflect(ignore)]` fields are dropped.
/// Complete failures will yield a [`ReflectCloneError`].
///
/// # Comparison with other reflection-cloning methods
///
/// Bevy offers two other ways to copy a reflected value, and *neither* is a good
/// general-purpose choice for reporting / debugging workflows.
///
/// [`PartialReflect::reflect_clone`] generates a direct, concrete clone of the value.
/// It keeps the real type but fails when any field is non-cloneable.
/// This is common when the type contains any `#[reflect(ignore)]` fields.
///
/// [`PartialReflect::to_dynamic`] instead builds a dynamic representation that simply omits
/// non-cloneable fields, so it succeeds for more types.
/// But it can panic on opaque values when cloning fails,
/// and sacrifices information about the type and its fields.
///
/// This method prefers the more faithful `reflect_clone` path, falling back to `to_dynamic` when necessary.
/// Opaque values have no dynamic form, so they will always return an error when `reflect_clone` fails.
// Upstreaming notes:
// - `clone_incomplete` should just be a method on `PartialReflect`
// - remember to cross-link from `PartialReflect::reflect_clone` and `PartialReflect::to_dynamic` for breadcrumbs
// - `to_dynamic` should be made to return a `Result` in the same PR that adds this method
pub fn clone_incomplete(
    reflected: &dyn PartialReflect,
) -> Result<Box<dyn PartialReflect>, ReflectCloneError> {
    match reflected.reflect_clone() {
        // Prefer a concrete clone to preserve data
        Ok(cloned) => Ok(cloned.into_partial_reflect()),
        // A concrete clone failed,
        // almost always because of a non-cloneable field such as `#[reflect(ignore)]`.
        // We should try to salvage a dynamic copy that simply omits those fields.
        Err(err) => match reflected.reflect_ref() {
            // Opaque values have no dynamic form so just return the error.
            ReflectRef::Opaque(_) => Err(err),
            // BUG: this will probably panic with if nested fields have unclonable opaque values.
            // Fixing to_dynamic to return a Result is much cleaner than working around it here,
            // so we should just do that during upstreaming.
            _ => Ok(reflected.to_dynamic()),
        },
    }
}

/// Reflects the value of the component identified by `type_id` on `entity`, formatting it for debugging.
///
/// Resources are stored as components on a dedicated backing entity, so this serves both
/// component and resource inspection.
///
/// Returns `"Dynamic Type"` when `type_id` is `None`,
/// and an "<Unreflectable: ...>" error string when reflection fails.
pub fn component_value_to_string(
    world: &World,
    entity: Entity,
    type_id: Option<core::any::TypeId>,
    full_type_names: bool,
) -> String {
    match type_id {
        Some(type_id) => match world.get_reflect(entity, type_id) {
            Ok(reflected) => {
                reflected_value_to_string(reflected.as_partial_reflect(), full_type_names)
            }
            Err(err) => format!("<Unreflectable: {err}>"),
        },
        None => "Dynamic Type".to_string(),
    }
}

/// Converts a reflected value to a string for debugging purposes.
// When upstreamed, this should be a method on `PartialReflect`,
// although much of it should be a `Display` impl on `ReflectRef`.
pub fn reflected_value_to_string(reflected: &dyn PartialReflect, full_type_names: bool) -> String {
    let reflect_ref = reflected.reflect_ref();
    match reflect_ref {
        ReflectRef::Struct(dyn_struct) => {
            pretty_print_reflected_struct(dyn_struct, full_type_names)
        }
        ReflectRef::TupleStruct(tuple_struct) => {
            pretty_print_reflected_tuple_struct(tuple_struct, full_type_names)
        }
        ReflectRef::Tuple(tuple) => pretty_print_reflected_tuple(tuple, full_type_names),
        ReflectRef::List(list) => pretty_print_reflected_list(list, full_type_names),
        ReflectRef::Array(array) => pretty_print_reflected_array(array, full_type_names),
        ReflectRef::Map(map) => pretty_print_reflected_map(map, full_type_names),
        ReflectRef::Set(set) => pretty_print_reflected_set(set, full_type_names),
        ReflectRef::Enum(dyn_enum) => pretty_print_reflected_enum(dyn_enum, full_type_names),
        ReflectRef::Opaque(opaque_partial_reflect) => {
            pretty_print_reflected_opaque(opaque_partial_reflect)
        }
    }
}

pub fn pretty_print_reflected_struct(dyn_struct: &dyn Struct, full_type_names: bool) -> String {
    let type_name = display_type_name(
        dyn_struct.get_represented_type_info(),
        "<Unknown Struct>",
        full_type_names,
    );

    let entries: Vec<String> = (0..dyn_struct.field_len())
        .map(|i| {
            let field_name = dyn_struct.name_at(i).unwrap_or("<Unknown Field>");
            let field_value = get_value_string(dyn_struct.field_at(i), full_type_names);
            format!("{field_name}: {field_value},")
        })
        .collect();

    format_block(&format!("{type_name} "), '{', &entries, '}')
}

pub fn pretty_print_reflected_tuple_struct(
    dyn_tuple_struct: &dyn TupleStruct,
    full_type_names: bool,
) -> String {
    let type_name = display_type_name(
        dyn_tuple_struct.get_represented_type_info(),
        "<Unknown TupleStruct>",
        full_type_names,
    );

    let entries: Vec<String> = (0..dyn_tuple_struct.field_len())
        .map(|i| {
            format!(
                "{},",
                get_value_string(dyn_tuple_struct.field(i), full_type_names)
            )
        })
        .collect();

    format_block(&type_name, '(', &entries, ')')
}

pub fn pretty_print_reflected_tuple(dyn_tuple: &dyn Tuple, full_type_names: bool) -> String {
    let entries: Vec<String> = (0..dyn_tuple.field_len())
        .map(|i| format!("{},", get_value_string(dyn_tuple.field(i), full_type_names)))
        .collect();

    format_block("", '(', &entries, ')')
}

pub fn pretty_print_reflected_list(dyn_list: &dyn List, full_type_names: bool) -> String {
    let entries: Vec<String> = (0..dyn_list.len())
        .map(|i| format!("{},", get_value_string(dyn_list.get(i), full_type_names)))
        .collect();

    format_block("", '[', &entries, ']')
}

pub fn pretty_print_reflected_array(dyn_array: &dyn Array, full_type_names: bool) -> String {
    let entries: Vec<String> = (0..dyn_array.len())
        .map(|i| format!("{},", get_value_string(dyn_array.get(i), full_type_names)))
        .collect();

    format_block("", '[', &entries, ']')
}

pub fn pretty_print_reflected_map(dyn_map: &dyn Map, full_type_names: bool) -> String {
    let entries: Vec<String> = dyn_map
        .iter()
        .map(|(key, value)| {
            let key = reflected_value_to_string(key, full_type_names);
            let value = reflected_value_to_string(value, full_type_names);
            format!("{key}: {value},")
        })
        .collect();

    format_block("", '{', &entries, '}')
}

pub fn pretty_print_reflected_set(dyn_set: &dyn Set, full_type_names: bool) -> String {
    let entries: Vec<String> = dyn_set
        .iter()
        .map(|element| format!("{},", reflected_value_to_string(element, full_type_names)))
        .collect();

    format_block("", '{', &entries, '}')
}

pub fn pretty_print_reflected_enum(dyn_enum: &dyn Enum, full_type_names: bool) -> String {
    let type_name = display_type_name(
        dyn_enum.get_represented_type_info(),
        "<Unknown Enum>",
        full_type_names,
    );
    let qualified = format!("{type_name}::{variant}", variant = dyn_enum.variant_name());

    match dyn_enum.variant_type() {
        VariantType::Struct => {
            let entries: Vec<String> = (0..dyn_enum.field_len())
                .map(|i| {
                    let field_name = dyn_enum.name_at(i).unwrap_or("<Unknown Field>");
                    let field_value = get_value_string(dyn_enum.field_at(i), full_type_names);
                    format!("{field_name}: {field_value},")
                })
                .collect();
            format_block(&format!("{qualified} "), '{', &entries, '}')
        }
        VariantType::Tuple => {
            let entries: Vec<String> = (0..dyn_enum.field_len())
                .map(|i| {
                    format!(
                        "{},",
                        get_value_string(dyn_enum.field_at(i), full_type_names)
                    )
                })
                .collect();
            format_block(&qualified, '(', &entries, ')')
        }
        VariantType::Unit => qualified,
    }
}

pub fn pretty_print_reflected_opaque(opaque_partial_reflect: &dyn PartialReflect) -> String {
    let debug = format!("{opaque_partial_reflect:?}");
    let trimmed = debug.trim();

    // Fast path return for single-line debug output
    if trimmed.len() == debug.len() && !trimmed.contains('\n') {
        return debug;
    }

    // A custom `Debug` may span lines;
    // indent so the value reads as one block nested under its label.
    let mut result = String::with_capacity(trimmed.len());
    push_indented(&mut result, trimmed, false);
    result
}

fn get_value_string(partial_reflect: Option<&dyn PartialReflect>, full_type_names: bool) -> String {
    if let Some(value) = partial_reflect {
        reflected_value_to_string(value, full_type_names)
    } else {
        String::from("<Unknown Value>")
    }
}

/// Display name for a reflected type, honoring `full_type_names`.
///
/// Uses `fallback` when no type information is available (e.g. for dynamic values).
fn display_type_name(
    type_info: Option<&TypeInfo>,
    fallback: &'static str,
    full_type_names: bool,
) -> String {
    match type_info {
        Some(info) => {
            let type_path = info.type_path();
            if full_type_names {
                type_path.to_string()
            } else {
                ShortName::from(type_path).to_string()
            }
        }
        None => fallback.to_string(),
    }
}

/// Appends `block` to `out`, indenting non-blank lines by two spaces.
///
/// No trailing newline is written, and blank lines are skipped.
///
/// When `indent_first` is false, the first line keeps its column.
/// This is useful when continuing already-written text (an inline value)
/// while later lines nest beneath it.
fn push_indented(out: &mut String, block: &str, indent_first: bool) {
    for (i, line) in block.lines().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        // A blank line would only contribute trailing whitespace.
        if line.trim().is_empty() {
            continue;
        }
        if i > 0 || indent_first {
            out.push_str("  ");
        }
        out.push_str(line);
    }
}

/// Wraps already-formatted `entries` in a delimited block, indenting each entry.
///
/// Multi-line entries are indented whole, so nested values stay aligned under their delimiters.
/// Empty blocks collapse to a single line: `{prefix}{open}{close}`.
fn format_block(prefix: &str, open: char, entries: &[String], close: char) -> String {
    if entries.is_empty() {
        return format!("{prefix}{open}{close}");
    }

    let prefix_count = prefix.len();
    let delimiter_count = 2;
    // Each entry has two spaces of indentation and a newline,
    // so add 3 to each entry's length to get a conservative estimate.
    let estimated_entry_count = entries.iter().map(|entry| entry.len() + 3).sum::<usize>();
    let estimated_capacity = prefix_count + delimiter_count + estimated_entry_count;

    let mut result = String::with_capacity(estimated_capacity);
    result.push_str(prefix);
    result.push(open);
    result.push('\n');

    for entry in entries {
        push_indented(&mut result, entry, true);
        result.push('\n');
    }

    result.push(close);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, HashSet};

    #[derive(Reflect)]
    struct Inner {
        a: u32,
        b: String,
    }

    #[derive(Reflect)]
    struct Outer {
        name: String,
        inner: Inner,
        list: Vec<i32>,
    }

    #[derive(Reflect)]
    struct EmptyStruct {}

    #[derive(Reflect)]
    struct Newtype(u32);

    #[derive(Reflect)]
    struct EmptyTupleStruct();

    #[derive(Reflect)]
    enum MyEnum {
        Unit,
        Tuple(u32, String),
        Struct { x: i32, inner: Inner },
        EmptyTuple(),
        EmptyStruct {},
    }

    fn pretty(value: &dyn PartialReflect) -> String {
        reflected_value_to_string(value, false)
    }

    #[test]
    fn opaque_values_use_debug() {
        assert_eq!(pretty(&42u32), "42");
        assert_eq!(pretty(&"hi".to_string()), "\"hi\"");
        assert_eq!(pretty(&true), "true");
    }

    #[test]
    fn flat_struct() {
        let value = Inner {
            a: 1,
            b: "two".to_string(),
        };
        assert_eq!(pretty(&value), "Inner {\n  a: 1,\n  b: \"two\",\n}");
    }

    #[test]
    fn empty_containers_collapse_to_one_line() {
        assert_eq!(pretty(&EmptyStruct {}), "EmptyStruct {}");
        assert_eq!(pretty(&EmptyTupleStruct()), "EmptyTupleStruct()");
        assert_eq!(pretty(&Vec::<i32>::new()), "[]");
        assert_eq!(pretty(&[0i32; 0]), "[]");
        assert_eq!(pretty(&()), "()");
        assert_eq!(pretty(&BTreeMap::<u32, u32>::new()), "{}");
        assert_eq!(pretty(&HashSet::<u32>::new()), "{}");
    }

    #[test]
    fn empty_enum_variants() {
        assert_eq!(pretty(&MyEnum::EmptyTuple()), "MyEnum::EmptyTuple()");
        assert_eq!(pretty(&MyEnum::EmptyStruct {}), "MyEnum::EmptyStruct {}");
    }

    #[test]
    fn newtype_struct() {
        assert_eq!(pretty(&Newtype(5)), "Newtype(\n  5,\n)");
    }

    #[test]
    fn list_of_scalars() {
        assert_eq!(pretty(&vec![1, 2, 3]), "[\n  1,\n  2,\n  3,\n]");
    }

    #[test]
    fn map_entries() {
        let mut map = BTreeMap::new();
        map.insert(1u32, "one".to_string());
        map.insert(2u32, "two".to_string());
        assert_eq!(pretty(&map), "{\n  1: \"one\",\n  2: \"two\",\n}");
    }

    #[test]
    fn enum_variants() {
        assert_eq!(pretty(&MyEnum::Unit), "MyEnum::Unit");
        assert_eq!(
            pretty(&MyEnum::Tuple(7, "t".to_string())),
            "MyEnum::Tuple(\n  7,\n  \"t\",\n)"
        );
    }

    #[test]
    fn nested_values_are_indented_per_level() {
        let value = Outer {
            name: "hello".to_string(),
            inner: Inner {
                a: 1,
                b: "two".to_string(),
            },
            list: vec![10, 20],
        };

        let expected = "\
Outer {
  name: \"hello\",
  inner: Inner {
    a: 1,
    b: \"two\",
  },
  list: [
    10,
    20,
  ],
}";
        assert_eq!(pretty(&value), expected);
    }

    #[test]
    fn deeply_nested_enum_struct_variant() {
        let value = MyEnum::Struct {
            x: -1,
            inner: Inner {
                a: 2,
                b: "q".to_string(),
            },
        };

        let expected = "\
MyEnum::Struct {
  x: -1,
  inner: Inner {
    a: 2,
    b: \"q\",
  },
}";
        assert_eq!(pretty(&value), expected);
    }

    #[test]
    fn full_type_names_uses_full_path() {
        let value = Inner {
            a: 1,
            b: "two".to_string(),
        };
        let rendered = reflected_value_to_string(&value, true);
        assert!(
            rendered.starts_with("feathers_inspector::reflection_tools::tests::Inner {"),
            "unexpected rendering: {rendered}"
        );
    }

    #[derive(Reflect, Clone)]
    #[reflect(opaque, Debug)]
    struct TrailingNewlineDebug;

    impl core::fmt::Debug for TrailingNewlineDebug {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            writeln!(f, "value")
        }
    }

    #[derive(Reflect)]
    struct HoldsTrailingNewline {
        op: TrailingNewlineDebug,
    }

    #[test]
    fn opaque_debug_trailing_newline_does_not_orphan_comma() {
        assert_eq!(pretty(&TrailingNewlineDebug), "value");
        assert_eq!(
            pretty(&HoldsTrailingNewline {
                op: TrailingNewlineDebug
            }),
            "HoldsTrailingNewline {\n  op: value,\n}"
        );
    }

    #[derive(Reflect, Clone)]
    #[reflect(opaque, Debug)]
    struct TrailingNewlineThenSpaces;

    impl core::fmt::Debug for TrailingNewlineThenSpaces {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "value\n  ")
        }
    }

    #[derive(Reflect)]
    struct HoldsTrailingNewlineThenSpaces {
        op: TrailingNewlineThenSpaces,
    }

    #[test]
    fn opaque_debug_trailing_whitespace_does_not_orphan_comma() {
        assert_eq!(pretty(&TrailingNewlineThenSpaces), "value");
        assert_eq!(
            pretty(&HoldsTrailingNewlineThenSpaces {
                op: TrailingNewlineThenSpaces
            }),
            "HoldsTrailingNewlineThenSpaces {\n  op: value,\n}"
        );
    }

    #[derive(Reflect, Clone)]
    #[reflect(opaque, Debug)]
    struct MultiLineDebug;

    impl core::fmt::Debug for MultiLineDebug {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "line1\nline2")
        }
    }

    #[derive(Reflect)]
    struct HoldsMultiLine {
        op: MultiLineDebug,
    }

    #[test]
    fn multi_line_opaque_value_indents_continuation_lines() {
        assert_eq!(pretty(&MultiLineDebug), "line1\n  line2");
        assert_eq!(
            pretty(&HoldsMultiLine { op: MultiLineDebug }),
            "HoldsMultiLine {\n  op: line1\n    line2,\n}"
        );
    }

    #[derive(Reflect, Clone)]
    #[reflect(opaque, Debug)]
    struct BlankInteriorLineDebug;

    impl core::fmt::Debug for BlankInteriorLineDebug {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "line1\n   \nline2")
        }
    }

    #[derive(Reflect)]
    struct HoldsBlankInterior {
        op: BlankInteriorLineDebug,
    }

    #[test]
    fn opaque_debug_blank_interior_line_carries_no_trailing_whitespace() {
        assert_eq!(pretty(&BlankInteriorLineDebug), "line1\n\n  line2");
        assert_eq!(
            pretty(&HoldsBlankInterior {
                op: BlankInteriorLineDebug
            }),
            "HoldsBlankInterior {\n  op: line1\n\n    line2,\n}"
        );
        let rendered = pretty(&HoldsBlankInterior {
            op: BlankInteriorLineDebug,
        });
        for line in rendered.lines() {
            assert_eq!(
                line.trim_end(),
                line,
                "line has trailing whitespace: {line:?}"
            );
        }
    }

    #[derive(Reflect, Clone)]
    #[reflect(opaque, Debug)]
    struct LeadingNewlineDebug;

    impl core::fmt::Debug for LeadingNewlineDebug {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "\nvalue")
        }
    }

    #[derive(Reflect)]
    struct HoldsLeadingNewline {
        op: LeadingNewlineDebug,
    }

    #[test]
    fn opaque_debug_leading_newline_does_not_leave_trailing_whitespace() {
        assert_eq!(pretty(&LeadingNewlineDebug), "value");
        assert_eq!(
            pretty(&HoldsLeadingNewline {
                op: LeadingNewlineDebug
            }),
            "HoldsLeadingNewline {\n  op: value,\n}"
        );
    }

    #[test]
    fn multi_line_struct_as_list_element() {
        let value = vec![
            Inner {
                a: 1,
                b: "x".to_string(),
            },
            Inner {
                a: 2,
                b: "y".to_string(),
            },
        ];
        let expected = "\
[
  Inner {
    a: 1,
    b: \"x\",
  },
  Inner {
    a: 2,
    b: \"y\",
  },
]";
        assert_eq!(pretty(&value), expected);
    }

    #[test]
    fn multi_line_struct_as_map_value() {
        let mut map = BTreeMap::new();
        map.insert(
            1u32,
            Inner {
                a: 10,
                b: "x".to_string(),
            },
        );
        let expected = "\
{
  1: Inner {
    a: 10,
    b: \"x\",
  },
}";
        assert_eq!(pretty(&map), expected);
    }
}
