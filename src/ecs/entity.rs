use crate::ecs::{component::Component, world::World};

#[derive(Clone, Copy)]
pub struct Entity(pub(crate) usize);

pub struct EntityWorld<'a> {
    pub(crate) world: &'a mut World,
    pub(crate) entity: Entity,
}

impl EntityWorld<'_> {
    pub fn insert<T: Component + 'static>(self, component: T) -> Self {
        self.world.add_component(self.entity, component);
        self
    }

    pub fn remove<T: Component + 'static>(self) -> Self {
        self.world.remove_component::<T>(self.entity);
        self
    }

    pub fn id(self) -> Entity {
        self.entity
    }

    pub fn despawn(self) {
        self.world.despawn(self.entity);
    }

    pub fn get_component<T: Component + 'static>(&self) -> Option<&T> {
        self.world.get_component::<T>(self.entity)
    }

    pub fn get_component_mut<T: Component + 'static>(&mut self) -> Option<&mut T> {
        self.world.get_component_mut::<T>(self.entity)
    }

    pub fn component<T: Component + 'static>(&self) -> &T {
        self.get_component::<T>().expect("Component not found")
    }

    pub fn component_mut<T: Component + 'static>(&mut self) -> &mut T {
        self.get_component_mut::<T>().expect("Component not found")
    }

    pub fn print_components(&self) {
        self.world.print_components(self.entity);
    }
}
