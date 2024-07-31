pub fn create_block_id_to_lines_map(all_events: &[rotext::BlendEvent]) -> String {
    let mut result = String::new();

    for ev in all_events.iter() {
        match ev {
            rotext::BlendEvent::ThematicBreak(data) => {
                write_id_and_line_range(
                    &mut result,
                    data.id.value(),
                    data.line_number,
                    data.line_number,
                );
            }
            rotext::BlendEvent::ExitBlock(data) => {
                write_id_and_line_range(
                    &mut result,
                    data.id.value(),
                    data.start_line_number,
                    data.end_line_number,
                );
            }
            _ => continue,
        }

        result.push(';');
    }

    if !result.is_empty() {
        result.pop().unwrap();
    }

    result
}

fn write_id_and_line_range(target: &mut String, id: usize, range_start: usize, range_end: usize) {
    write_usize(target, id);
    target.push(':');
    write_usize(target, range_start);
    target.push('-');
    write_usize(target, range_end);
}

fn write_usize(target: &mut String, n: usize) {
    let mut buffer = itoa::Buffer::new();
    target.push_str(buffer.format(n));
}
