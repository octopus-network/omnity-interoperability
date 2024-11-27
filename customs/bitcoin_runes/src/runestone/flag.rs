pub(super) enum Flag {
    Etching = 0,
    Terms = 1,
    Turbo = 2,
}

impl Flag {
    pub(super) fn mask(self) -> u128 {
        1 << self as u128
    }

    pub(super) fn set(self, flags: &mut u128) {
      *flags |= self.mask()
    }
}