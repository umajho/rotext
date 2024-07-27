use std::{fs, path::PathBuf};

fn main() {
    divan::main();
}

#[divan::bench(sample_size = 10_000)]
fn parsing_getting_started(bencher: divan::Bencher) {
    let file_path = {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../../docs/rotext入门-new.rotext");

        path
    };

    let file_content = fs::read_to_string(file_path).unwrap();

    bencher.bench(|| {
        rotext::parse(file_content.as_bytes());
    })
}

#[divan::bench(sample_size = 10)]
fn rendering_getting_started(bencher: divan::Bencher) {
    let file_path = {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../../docs/rotext入门-new.rotext");

        path
    };

    let file_content = fs::read_to_string(file_path).unwrap();

    bencher
        .with_inputs(|| rotext::parse(file_content.as_bytes()))
        .bench_refs(|events| {
            rotext::render_to_html(
                file_content.as_bytes(),
                events,
                rotext::RenderToHTMLOptions {
                    initial_output_string_capacity: file_content.len() * 3,
                },
            );
        })
}
