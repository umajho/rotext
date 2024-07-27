use std::{fs, path::PathBuf, sync::LazyLock};

fn main() {
    divan::main();
}

static CONTENT: LazyLock<String> = LazyLock::new(|| read_doc("rotext入门-new.rotext"));

#[divan::bench(sample_size = 10_000)]
fn parsing_getting_started(bencher: divan::Bencher) {
    let file_content = CONTENT.clone();

    bencher.bench(|| {
        rotext::parse(file_content.as_bytes());
    })
}

#[divan::bench(sample_size = 10)]
fn rendering_getting_started_using_string(bencher: divan::Bencher) {
    let file_content = read_doc("rotext入门-new.rotext");

    bencher
        .with_inputs(|| rotext::parse(file_content.as_bytes()))
        .bench_refs(|events| {
            rotext::rendering::using_string::render_to_html(
                file_content.as_bytes(),
                events,
                rotext::RenderToHTMLOptions {
                    initial_output_string_capacity: file_content.len() * 3,
                },
            );
        })
}

#[divan::bench(sample_size = 10)]
fn rendering_getting_started_using_vec_u8(bencher: divan::Bencher) {
    let file_content = read_doc("rotext入门-new.rotext");

    bencher
        .with_inputs(|| rotext::parse(file_content.as_bytes()))
        .bench_refs(|events| {
            rotext::rendering::using_vec_u8::render_to_html(
                file_content.as_bytes(),
                events,
                rotext::RenderToHTMLOptions {
                    initial_output_string_capacity: file_content.len() * 3,
                },
            );
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
