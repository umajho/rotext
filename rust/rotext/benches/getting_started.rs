use std::{fs, path::PathBuf, sync::LazyLock};

fn main() {
    divan::main();
}

static CONTENT: LazyLock<String> = LazyLock::new(|| read_doc("rotext入门.rotext"));

fn compile_and_execute(input: &[u8], events: &[rotext::Event]) {
    // 由于 `new_demo_instance_for_test` 位于 `test` 特性旗帜之后，这里会误报错误。
    let tag_name_map = rotext::TagNameMap::new_demo_instance_for_test();

    let compile_opts = rotext::CompileOption {
        restrictions: rotext::CompileRestrictions {
            max_call_depth_in_document: 100,
        },
    };
    let compiled = rotext::compile(input, events, &compile_opts).unwrap();

    let exec_opts = rotext::ExecuteOptions {
        tag_name_map: &tag_name_map,
        // 由于 `new_demo_block_extension_map_for_test` 位于 `test` 特性旗帜之后，这里会误报错误。
        block_extension_map: &rotext::executing::extensions::new_demo_block_extension_map_for_test(
        ),
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
