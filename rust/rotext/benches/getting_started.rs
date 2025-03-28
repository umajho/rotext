use std::{fs, path::PathBuf, sync::LazyLock};

fn main() {
    divan::main();
}

static CONTENT: LazyLock<String> = LazyLock::new(|| read_doc("rotext入门.rotext"));

fn render(input: &[u8], events: &[rotext::Event]) {
    let tag_name_map = rotext::TagNameMap::default();

    let compile_opts = rotext::CompileOption {
        restrictions: rotext::CompileRestrictions {
            max_call_depth_in_document: 100,
        },
        tag_name_map: &tag_name_map,
        #[cfg(feature = "block-id")]
        should_include_block_ids: true,
    };
    let compiled = rotext::compile(input, events, &compile_opts).unwrap();

    let render_opts = rotext::RenderOptions {
        tag_name_map: &tag_name_map,
    };
    rotext::render(&compiled, render_opts);
}

#[divan::bench(sample_size = 10)]
fn parsing(bencher: divan::Bencher) {
    let file_content = CONTENT.clone();

    bencher.bench(|| {
        rotext::parse(file_content.as_bytes()).for_each(drop);
    })
}

#[divan::bench(sample_size = 10)]
fn rendering(bencher: divan::Bencher) {
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
            render(file_content, events);
        })
}

#[divan::bench(sample_size = 10)]
fn parsing_and_rendering(bencher: divan::Bencher) {
    let file_content = CONTENT.clone();
    let file_content = file_content.as_bytes();

    bencher.bench(|| {
        let events = rotext::parse(file_content)
            .map(Result::unwrap)
            .collect::<Vec<_>>();
        render(file_content, &events);
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
