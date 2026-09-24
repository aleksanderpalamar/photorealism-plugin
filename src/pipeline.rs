pub enum Pipeline<R> {
    Missing,
    Ready(R),
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Demand {
    Create,
    Reuse,
    Skip,
}

impl<R> Pipeline<R> {
    pub fn demand(&self, matches: impl FnOnce(&R) -> bool) -> Demand {
        match self {
            Self::Missing => Demand::Create,
            Self::Failed => Demand::Skip,
            Self::Ready(resource) => {
                if matches(resource) {
                    Demand::Reuse
                } else {
                    Demand::Create
                }
            }
        }
    }

    pub fn store(&mut self, created: Option<R>) {
        *self = match created {
            Some(resource) => Self::Ready(resource),
            None => Self::Failed,
        };
    }

    pub fn get(&self) -> Option<&R> {
        match self {
            Self::Ready(resource) => Some(resource),
            Self::Missing | Self::Failed => None,
        }
    }

    pub fn ready(&mut self) -> Option<&mut R> {
        match self {
            Self::Ready(resource) => Some(resource),
            Self::Missing | Self::Failed => None,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::Missing;
    }
}

#[cfg(test)]
#[path = "pipeline_tests.rs"]
mod tests;
