use crate::{AliasId, DeclId, ModuleId};

pub enum Exportable {
    Decl { name: Vec<u8>, id: DeclId },
    Alias { name: Vec<u8>, id: AliasId },
    Module { name: Vec<u8>, id: ModuleId },
}
