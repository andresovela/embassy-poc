use crate::Ms;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Event {
    Release(Kind),
    Press(Kind),
    Hold(Ms),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Kind {
    Raw,
    Single(Length),
    Double(Length),
    Triple(Length),
    Repeated(Length, u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Length {
    Short,
    Medium,
    Long,
    VeryLong,
}

impl Event {
    pub(crate) fn into_release(self) -> Self {
        match self {
            Event::Release(_) => self,
            Event::Press(k) => Event::Release(k),
            Event::Hold(k, _) => Event::Release(k),
        }
    }

    pub(crate) fn with_length(self, length: Length) -> Self {
        match self {
            Event::Release(k) => Event::Release(self.kind().with_length(length)),
            Event::Press(k) => Event::Press(self.kind().with_length(length)),
            Event::Hold(k, count) => Event::Hold(self.kind().with_length(length), count),
        }
    }

    fn kind(&self) -> Kind {
        match self {
            Event::Release(k) => k.clone(),
            Event::Press(k) => k.clone(),
            Event::Hold(k, _) => k.clone(),
        }
    }
}

impl Kind {
    pub(crate) fn with_length(self, length: Length) -> Self {
        match self {
            Kind::Raw => panic!("Cannot add length to `Kind::Raw`"),
            Kind::Single(l) => Kind::Single(length),
            Kind::Double(l) => Kind::Double(length),
            Kind::Triple(l) => Kind::Triple(length),
            Kind::Repeated(l, count) => Kind::Repeated(length, count.clone()),
        }
    }
}
