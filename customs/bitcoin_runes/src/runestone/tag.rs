use super::varint;

#[derive(Copy, Clone, Debug)]
pub(super) enum Tag {
    Body = 0,
    Mint = 20,
}

impl Tag {
    pub(super) fn encode<const N: usize>(self, values: [u128; N], payload: &mut Vec<u8>) {
        for value in values {
            varint::encode_to_vec(self.into(), payload);
            varint::encode_to_vec(value, payload);
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
