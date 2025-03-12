#[cfg(debug_assertions)]
pub fn render_events_in_debug_format(input: &[u8], all_events: &Vec<rotext::Event>) -> String {
    let mut output = "".to_string();

    for event in all_events {
        let content = event.clone().content_u8_slice(input);
        let content = content.map(|content| std::str::from_utf8(content).unwrap());

        output.push_str(&format!("{:?} {:?}\n", event, content));
    }

    output
}
