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
            Event::Hold(_) => panic!("Cannot convert `Event::Hold` into `Event::Release`"),
        }
    }

    pub(crate) fn with_length(self, length: Length) -> Self {
        match self {
            Event::Release(_) => Event::Release(self.kind().with_length(length)),
            Event::Press(_) => Event::Press(self.kind().with_length(length)),
            Event::Hold(_) => panic!("Cannot add length to `Event::Hold`"),
        }
    }

    fn kind(&self) -> Kind {
        match self {
            Event::Release(k) => k.clone(),
            Event::Press(k) => k.clone(),
            Event::Hold(_) => unreachable!(),
        }
    }
}

impl Kind {
    pub(crate) fn with_length(self, length: Length) -> Self {
        match self {
            Kind::Raw => panic!("Cannot add length to `Kind::Raw`"),
            Kind::Single(_) => Kind::Single(length),
            Kind::Double(_) => Kind::Double(length),
            Kind::Triple(_) => Kind::Triple(length),
            Kind::Repeated(_, count) => Kind::Repeated(length, count.clone()),
        }
    }
}
