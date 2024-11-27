use super::varint;

#[derive(Copy, Clone, Debug)]
pub(super) enum Tag {
    Body = 0,
    Flags = 2,
    Rune = 4,
    Premine = 6,
    Cap = 8,
    Amount = 10,
    HeightStart = 12,
    HeightEnd = 14,
    OffsetStart = 16,
    OffsetEnd = 18,
    Mint = 20, 
    Divisibility = 1,
    Spacers = 3,
    Symbol = 5,
}

impl Tag {
    pub(super) fn encode<const N: usize>(self, values: [u128; N], payload: &mut Vec<u8>) {
        for value in values {
            varint::encode_to_vec(self.into(), payload);
            varint::encode_to_vec(value, payload);
        }
    }

    pub(super) fn encode_option<T: Into<u128>>(self, value: Option<T>, payload: &mut Vec<u8>) {
        if let Some(value) = value {
          self.encode([value.into()], payload)
        }
      }
}

impl From<Tag> for u128 {
    fn from(tag: Tag) -> Self {
        tag as u128
    }
}

impl PartialEq<u128> for Tag {
    fn eq(&self, other: &u128) -> bool {
        u128::from(*self) == *other
    }
}
