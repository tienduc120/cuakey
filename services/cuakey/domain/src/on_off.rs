/// A switch that is on or off. It stands in for `bool` wherever a function or a struct would take several flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OnOff {
    On,
    Off,
}

impl OnOff {
    pub const fn is_on(self) -> bool {
        matches!(self, Self::On)
    }
}
