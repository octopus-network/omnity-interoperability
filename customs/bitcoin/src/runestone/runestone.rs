use super::varint;
use bitcoin::blockdata::{constants, opcodes, script};
use serde::Serialize;

#[derive(Copy, Clone, Debug)]
pub(super) enum Tag {
    Body = 0,
}

impl From<Tag> for u128 {
    fn from(tag: Tag) -> Self {
        tag as u128
    }
}

#[derive(Default, Serialize, Debug, PartialEq, Copy, Clone)]
pub struct Edict {
    pub id: u128,
    pub amount: u128,
    pub output: u128,
}

pub struct Runestone {
    pub edicts: Vec<Edict>,
}

impl Runestone {
    pub fn encipher(&self) -> [u8; 20] {
        assert!(!self.edicts.is_empty());

        let mut payload = Vec::new();
        varint::encode_to_vec(Tag::Body.into(), &mut payload);

        let mut edicts = self.edicts.clone();
        edicts.sort_by_key(|edict| edict.id);

        let mut id = 0;
        for edict in edicts {
            varint::encode_to_vec(edict.id - id, &mut payload);
            varint::encode_to_vec(edict.amount, &mut payload);
            varint::encode_to_vec(edict.output, &mut payload);
            id = edict.id;
        }

        let mut builder = script::Builder::new()
            .push_opcode(opcodes::all::OP_RETURN)
            .push_slice(b"RUNE_TEST");

        for chunk in payload.chunks(constants::MAX_SCRIPT_ELEMENT_SIZE) {
            let push = chunk.try_into().unwrap();
            builder = builder.push_slice(push);
        }

        builder
            .into_script()
            .script_hash()
            .to_vec()
            .as_slice()
            .try_into()
            .expect("slice with incorrect length")
    }
}
