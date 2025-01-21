use bitcoin::blockdata::{constants, opcodes, script};
use bitcoin::script::PushBytesBuf;
use serde::Serialize;

use omnity_types::rune_id::RuneId;

use crate::runestone::tag::Tag;

use super::varint;

const MAGIC_NUMBER: opcodes::Opcode = opcodes::all::OP_PUSHNUM_13;

#[derive(Default, Serialize, Debug, PartialEq, Copy, Clone)]
pub struct Edict {
    pub id: RuneId,
    pub amount: u128,
    pub output: u32,
}

#[derive(Default)]
pub struct Runestone {
    pub edicts: Vec<Edict>,
    pub mint: Option<RuneId>,
}

impl Runestone {
    pub fn encipher(&self) -> Vec<u8> {
        assert!(!self.edicts.is_empty() || self.mint.is_some());

        let mut payload = Vec::new();

        if let Some(RuneId { block, tx }) = self.mint {
            Tag::Mint.encode([block.into(), tx.into()], &mut payload);
        }

        if !self.edicts.is_empty() {
            varint::encode_to_vec(Tag::Body.into(), &mut payload);

            let mut edicts = self.edicts.clone();
            edicts.sort_by_key(|edict| edict.id);

            let mut previous = RuneId::default();
            for edict in edicts {
                let (block, tx) = previous.delta(edict.id).unwrap();
                varint::encode_to_vec(block, &mut payload);
                varint::encode_to_vec(tx, &mut payload);
                varint::encode_to_vec(edict.amount, &mut payload);
                varint::encode_to_vec(edict.output.into(), &mut payload);
                previous = edict.id;
            }
        }

        let mut builder = script::Builder::new()
            .push_opcode(opcodes::all::OP_RETURN)
            .push_opcode(MAGIC_NUMBER);

        for chunk in payload.chunks(constants::MAX_SCRIPT_ELEMENT_SIZE) {
            let mut v = chunk.to_vec();
            let mut push = PushBytesBuf::new();
            push.extend_from_slice(v.as_mut_slice()).unwrap();
            builder = builder.push_slice(push);
        }

        builder.into_script().to_bytes()
    }
}
