#[cfg(debug_assertions)]
pub fn render_events_in_debug_format(input: &[u8], all_events: &Vec<rotext::BlendEvent>) -> String {
    let mut output = "".to_string();

    for event in all_events {
        // output.push_str(&format!("{:?}\n", event));
        output.push_str(&format!(
            "{:?} {:?}\n",
            event,
            rotext::Event::from(event.clone()).content(input)
        ));
    }

    output
}
