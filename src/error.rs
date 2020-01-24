use x86_64::structures::paging::mapper::MapToError;

pub enum Error {
    OutOfFrames,
    MapTo(MapToError),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::OutOfFrames => write!(f, "Attempted to map a frame but no frames were available"),
            Self::MapTo(inner) => write!(f, "{:?}", inner),
        }
    }
}

impl core::fmt::Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self)
    }
}

impl From<MapToError> for Error {
    fn from(other: MapToError) -> Self {
        Self::MapTo(other)
    }
}