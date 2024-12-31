use rotext::rendering::TagNameMap;

pub fn new_tag_name_map_from_str(raw: &str) -> TagNameMap {
    let mut items = raw.as_bytes().split(|x| *x == 0);

    let code_block = items.next().unwrap();
    let ref_link = items.next().unwrap();
    let dicexp = items.next().unwrap();
    let wiki_link = items.next().unwrap();

    TagNameMap {
        code_block,
        ref_link,
        dicexp,
        wiki_link,
    }
}
