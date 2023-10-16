use crate::property::SharedProperty;

pub type BoolProperty = SharedProperty<bool>;

impl From<bool> for BoolProperty {
    fn from(value: bool) -> Self {
        Self::from_value(value)
    }
}

impl From<&BoolProperty> for BoolProperty{
    fn from(value: &BoolProperty) -> Self {
        value.clone()
    }
}