use crate::item::Alignment;
use crate::property::SharedProperty;

pub type AlignmentProperty = SharedProperty<Alignment>;

impl From<Alignment> for AlignmentProperty{
    fn from(alignment: Alignment) -> Self{
        Self::from_value(alignment)
    }
}