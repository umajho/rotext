use std::{fs, path::PathBuf, sync::LazyLock};

fn main() {
    divan::main();
}

static CONTENT: LazyLock<String> = LazyLock::new(|| read_doc("rotext入门.rotext"));

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

    bencher
        .with_inputs(|| {
            rotext::parse(file_content.as_bytes())
                .collect::<Vec<_>>()
                .into_iter()
                .map(Result::unwrap)
        })
        .bench_refs(|events| {
            let renderer = rotext::HtmlRenderer::new(
                file_content.as_bytes(),
                rotext::NewHtmlRendererOptions {
                    tag_name_map: Default::default(),
                    initial_output_string_capacity: file_content.len() * 3,
                    #[cfg(feature = "block-id")]
                    should_include_block_ids: true,
                },
            );
            renderer.render_u8_vec(events);
        })
}

#[divan::bench(sample_size = 10)]
fn parsing_and_rendering(bencher: divan::Bencher) {
    let file_content = CONTENT.clone();

    bencher.bench(|| {
        let events = rotext::parse(file_content.as_bytes()).map(Result::unwrap);
        let renderer = rotext::HtmlRenderer::new(
            file_content.as_bytes(),
            rotext::NewHtmlRendererOptions {
                tag_name_map: Default::default(),
                initial_output_string_capacity: file_content.len() * 3,
                #[cfg(feature = "block-id")]
                should_include_block_ids: true,
            },
        );
        renderer.render_u8_vec(events);
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
