use std::collections::HashMap;

pub type BlockIdToLInesMap = HashMap<usize, (usize, usize)>;

pub fn create_block_id_to_lines_map(
    all_events: &[rotext::Event],
) -> HashMap<usize, (usize, usize)> {
    let mut result = HashMap::new();

    for ev in all_events.iter() {
        match ev {
            rotext::Event::ThematicBreak(data) => {
                result.insert(data.id.value(), (data.line.value(), data.line.value()));
            }
            rotext::Event::ExitBlock(data) => {
                result.insert(
                    data.id.value(),
                    (data.start_line.value(), data.end_line.value()),
                );
            }
            _ => continue,
        }
    }

    result
}
