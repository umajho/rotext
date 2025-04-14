use rotext::TagNameMap;

#[derive(Debug, serde::Deserialize)]
pub struct TagNameMapInput {
    pub block_call_error: String,
    pub inline_call_error: String,

    pub code_block: String,

    pub ref_link: String,
    pub dicexp: String,
    pub wiki_link: String,
}

impl TagNameMapInput {
    pub fn convert(&self) -> TagNameMap {
        TagNameMap {
            block_call_error: self.block_call_error.as_bytes(),
            inline_call_error: self.inline_call_error.as_bytes(),
            code_block: self.code_block.as_bytes(),

            ref_link: self.ref_link.as_bytes(),
            dicexp: self.dicexp.as_bytes(),
            wiki_link: self.wiki_link.as_bytes(),
        }
    }
}
