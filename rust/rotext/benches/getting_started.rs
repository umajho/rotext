use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    sync::LazyLock,
};

use rotext::executing::extensions::{
    Extension, ExtensionElementMapper, ExtensionElementMapperParameter,
    ExtensionElementMapperParameterMappingTo, ExtensionElementMapperVerbatimParameter,
    ParameterWrapper,
};

fn main() {
    divan::main();
}

static CONTENT: LazyLock<String> = LazyLock::new(|| read_doc("rotext入门.rotext"));

fn compile_and_execute(input: &[u8], events: &[rotext::Event]) {
    let tag_name_map = rotext::TagNameMap::new_demo_instance_for_test();

    let compile_opts = rotext::CompileOption {
        restrictions: rotext::CompileRestrictions {
            max_call_depth_in_document: 100,
        },
    };
    let compiled = rotext::compile(input, events, &compile_opts).unwrap();

    let exec_opts = rotext::ExecuteOptions {
        tag_name_map: &tag_name_map,
        block_extension_map: &make_demo_block_extension_map(),
        #[cfg(feature = "block-id")]
        should_include_block_ids: true,
    };
    rotext::execute(input, events, &compiled, &exec_opts);
}

#[divan::bench(sample_size = 10)]
fn parsing(bencher: divan::Bencher) {
    let file_content = CONTENT.clone();

    bencher.bench(|| {
        rotext::parse(file_content.as_bytes()).for_each(drop);
    })
}

#[divan::bench(sample_size = 10)]
fn compiling_and_executing(bencher: divan::Bencher) {
    let file_content = CONTENT.clone();
    let file_content = file_content.as_bytes();

    bencher
        .with_inputs(|| {
            rotext::parse(file_content)
                .collect::<Vec<_>>()
                .into_iter()
                .map(Result::unwrap)
                .collect::<Vec<_>>()
        })
        .bench_refs(|events| {
            compile_and_execute(file_content, events);
        })
}

#[divan::bench(sample_size = 10)]
fn parsing_compiling_and_executing(bencher: divan::Bencher) {
    let file_content = CONTENT.clone();
    let file_content = file_content.as_bytes();

    bencher.bench(|| {
        let events = rotext::parse(file_content)
            .map(Result::unwrap)
            .collect::<Vec<_>>();
        compile_and_execute(file_content, &events);
    })
}

fn read_doc(name: &'static str) -> String {
    let file_path = {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../../docs");
        path.push(name);

        path
    };

    fs::read_to_string(file_path).unwrap()
}

fn make_demo_block_extension_map() -> HashMap<&'static [u8], Extension<'static>> {
    let mut map: HashMap<&'static [u8], Extension<'static>> = HashMap::new();

    map.insert(
        b"Div",
        Extension::ElementMapper(Box::new(ExtensionElementMapper {
            tag_name: b"div",
            variant: None,
            parameters: HashMap::new(),
            required_parameters: HashSet::new(),
            verbatim_parameters: HashMap::new(),
            required_verbatim_parameters: HashSet::new(),
        })),
    );

    map.insert(
        b"Collapse",
        Extension::ElementMapper(Box::new(ExtensionElementMapper {
            tag_name: b"x-collapse",
            variant: None,
            parameters: {
                let mut map: HashMap<
                    &'static [u8],
                    ParameterWrapper<ExtensionElementMapperParameter>,
                > = HashMap::new();
                map.insert(
                    b"1",
                    ParameterWrapper::Real(ExtensionElementMapperParameter {
                        mapping_to: ExtensionElementMapperParameterMappingTo::UnnamedSlot,
                    }),
                );
                map
            },
            required_parameters: {
                let mut set: HashSet<&'static [u8]> = HashSet::new();
                set.insert(b"1");
                set
            },
            verbatim_parameters: {
                let mut map: HashMap<
                    &'static [u8],
                    ParameterWrapper<ExtensionElementMapperVerbatimParameter>,
                > = HashMap::new();
                map.insert(
                    b"title",
                    ParameterWrapper::Real(ExtensionElementMapperVerbatimParameter {
                        mapping_to_attribute: b"title",
                    }),
                );
                map.insert("标题".as_bytes(), ParameterWrapper::Alias(b"title"));
                map.insert(
                    b"open",
                    ParameterWrapper::Real(ExtensionElementMapperVerbatimParameter {
                        mapping_to_attribute: b"open-by-default",
                    }),
                );
                map.insert("展开".as_bytes(), ParameterWrapper::Alias(b"open"));
                map
            },
            required_verbatim_parameters: HashSet::new(),
        })),
    );
    map.insert("折叠".as_bytes(), Extension::Alias { to: b"Collapse" });
    for (name, variant, alias) in [
        (&b"Note"[..], &b"note"[..], "注".as_bytes()),
        (b"Tip", b"tip", "提示".as_bytes()),
        (b"Important", b"important", "重要".as_bytes()),
        (b"Warning", b"warning", "警告".as_bytes()),
        (b"Caution", b"caution", "当心".as_bytes()),
    ] {
        map.insert(
            name,
            Extension::ElementMapper(Box::new(ExtensionElementMapper {
                tag_name: b"x-callout",
                variant: Some(variant),
                parameters: {
                    let mut map: HashMap<
                        &'static [u8],
                        ParameterWrapper<ExtensionElementMapperParameter>,
                    > = HashMap::new();
                    map.insert(
                        b"1",
                        ParameterWrapper::Real(ExtensionElementMapperParameter {
                            mapping_to: ExtensionElementMapperParameterMappingTo::UnnamedSlot,
                        }),
                    );
                    map
                },
                required_parameters: {
                    let mut set: HashSet<&'static [u8]> = HashSet::new();
                    set.insert(b"1");
                    set
                },
                verbatim_parameters: HashMap::new(),
                required_verbatim_parameters: HashSet::new(),
            })),
        );
        map.insert(alias, Extension::Alias { to: name });
    }

    map
}
