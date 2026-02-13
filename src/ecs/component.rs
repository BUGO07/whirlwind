use std::{any::Any, fmt::Debug};

pub trait Component: Any + Debug {}

impl dyn Component {
    pub(crate) fn downcast_ref<T: Any>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref()
    }
    pub(crate) fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        (self as &mut dyn Any).downcast_mut()
    }
}
