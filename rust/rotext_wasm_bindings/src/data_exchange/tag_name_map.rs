use rotext::TagNameMap;

#[derive(serde::Deserialize)]
pub struct TagNameMapInput {
    pub block_call_error: String,

    pub code_block: String,

    pub ref_link: String,
    pub dicexp: String,
    pub wiki_link: String,
}

impl TagNameMapInput {
    pub fn to_tag_name_map(&self) -> TagNameMap {
        TagNameMap {
            block_call_error: self.block_call_error.as_bytes(),
            code_block: self.code_block.as_bytes(),

            ref_link: self.ref_link.as_bytes(),
            dicexp: self.dicexp.as_bytes(),
            wiki_link: self.wiki_link.as_bytes(),
        }
    }
}
