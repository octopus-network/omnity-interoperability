use crate::runestone::{tag::Tag, flag::Flag};

use super::varint;
use bitcoin::blockdata::{constants, opcodes, script};
use omnity_types::rune_id::{RuneId, Etching};
use serde::Serialize;

const MAGIC_NUMBER: opcodes::All = opcodes::all::OP_PUSHNUM_13;

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
    pub etching: Option<Etching>,
}

impl Runestone {
    pub fn encipher(&self) -> Vec<u8> {
        assert!(!self.edicts.is_empty() || self.mint.is_some() || self.etching.is_some());

        let mut payload = Vec::new();

        if let Some(etching) = &self.etching {
            let mut flags = 0;
            Flag::Etching.set(&mut flags);
      
            if etching.terms.is_some() {
              Flag::Terms.set(&mut flags);
            }
      
            if etching.turbo {
              Flag::Turbo.set(&mut flags);
            }
      
            Tag::Flags.encode([flags], &mut payload);
      
            Tag::Rune.encode_option(etching.rune.map(|rune| rune.0), &mut payload);
            Tag::Divisibility.encode_option(etching.divisibility, &mut payload);
            Tag::Spacers.encode_option(etching.spacers, &mut payload);
            Tag::Symbol.encode_option(etching.symbol, &mut payload);
            Tag::Premine.encode_option(etching.premine, &mut payload);
      
            if let Some(terms) = etching.terms {
              Tag::Amount.encode_option(terms.amount, &mut payload);
              Tag::Cap.encode_option(terms.cap, &mut payload);
              Tag::HeightStart.encode_option(terms.height.0, &mut payload);
              Tag::HeightEnd.encode_option(terms.height.1, &mut payload);
              Tag::OffsetStart.encode_option(terms.offset.0, &mut payload);
              Tag::OffsetEnd.encode_option(terms.offset.1, &mut payload);
            }
        }

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
            let push = chunk.try_into().unwrap();
            builder = builder.push_slice(push);
        }

        builder.into_script().to_bytes()
    }
}
