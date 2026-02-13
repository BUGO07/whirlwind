use wgpu::naga::FastHashMap;

use crate::ecs::{
    component::Component,
    entity::{Entity, EntityWorld},
};

type EntityComponents = Option<Box<dyn Component>>;
type SystemFn = fn(&mut World);

#[derive(Default)]
pub struct World {
    components: FastHashMap<&'static str, Vec<EntityComponents>>,
    resources: FastHashMap<&'static str, Box<dyn Component>>,
    schedules: FastHashMap<&'static str, Vec<SystemFn>>,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_component<T: Component + 'static>(&mut self) {
        let type_name = std::any::type_name::<T>();
        let len = self.components.values().next().map_or(0, |v| v.len());
        self.components
            .insert(type_name, (0..len).map(|_| None).collect());
    }

    pub fn init_resource<T: Component + Default + 'static>(&mut self) {
        let type_name = std::any::type_name::<T>();
        self.resources.insert(type_name, Box::new(T::default()));
    }

    pub fn insert_resource<T: Component + 'static>(&mut self, resource: T) {
        let type_name = std::any::type_name::<T>();
        self.resources.insert(type_name, Box::new(resource));
    }

    pub fn get_resource<T: Component + 'static>(&self) -> Option<&T> {
        let type_name = std::any::type_name::<T>();
        self.resources.get(type_name)?.as_ref().downcast_ref::<T>()
    }

    pub fn get_resource_mut<T: Component + 'static>(&mut self) -> Option<&mut T> {
        let type_name = std::any::type_name::<T>();
        self.resources
            .get_mut(type_name)?
            .as_mut()
            .downcast_mut::<T>()
    }

    pub fn resource<T: Component + 'static>(&self) -> &T {
        self.get_resource::<T>().expect("Resource not found")
    }

    pub fn resource_mut<T: Component + 'static>(&mut self) -> &mut T {
        self.get_resource_mut::<T>().expect("Resource not found")
    }

    pub fn print_resources(&self) {
        for resource in self.resources.values() {
            println!("Resource: {:?}", resource);
        }
    }

    pub fn spawn(&'_ mut self) -> EntityWorld<'_> {
        let id = self.components.values().next().map_or(0, |v| v.len());
        for components in self.components.values_mut() {
            components.push(None);
        }
        EntityWorld {
            world: self,
            entity: Entity(id),
        }
    }

    pub fn despawn(&mut self, entity: Entity) {
        for components in self.components.values_mut() {
            if let Some(component) = components.get_mut(entity.0) {
                *component = None;
            }
        }
    }

    pub fn print_entities(&self) {
        for (type_name, components) in &self.components {
            for (index, component) in components.iter().enumerate() {
                if let Some(component) = component.as_ref() {
                    println!(
                        "Entity {} has component {}: {:?}",
                        index, type_name, component
                    );
                }
            }
        }
    }

    pub fn add_component<T: Component + 'static>(&mut self, entity: Entity, component: T) {
        let type_name = std::any::type_name::<T>();
        if let Some(components) = self.components.get_mut(type_name) {
            components[entity.0] = Some(Box::new(component));
        } else {
            self.register_component::<T>();
            self.add_component(entity, component);
        }
    }

    pub fn remove_component<T: Component + 'static>(&mut self, entity: Entity) {
        let type_name = std::any::type_name::<T>();
        if let Some(components) = self.components.get_mut(type_name) {
            components[entity.0] = None;
        } else {
            self.register_component::<T>();
        }
    }

    pub fn get_component<T: Component + 'static>(&self, entity: Entity) -> Option<&T> {
        let type_name = std::any::type_name::<T>();
        self.components
            .get(type_name)?
            .get(entity.0)?
            .as_ref()?
            .downcast_ref::<T>()
    }

    pub fn get_component_mut<T: Component + 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_name = std::any::type_name::<T>();
        self.components
            .get_mut(type_name)?
            .get_mut(entity.0)?
            .as_mut()?
            .downcast_mut::<T>()
    }

    pub fn print_components(&self, entity: Entity) {
        for components in self.components.values() {
            if let Some(component) = components.get(entity.0).and_then(|c| c.as_ref()) {
                println!("Entity {} has component: {:?}", entity.0, component);
            }
        }
    }

    pub fn query<T: Component + 'static>(&self) -> Vec<(Entity, &T)> {
        let type_name = std::any::type_name::<T>();
        if let Some(components) = self.components.get(type_name) {
            components
                .iter()
                .enumerate()
                .filter_map(|(index, component)| {
                    component
                        .as_ref()
                        .and_then(|c| c.downcast_ref::<T>())
                        .map(|c| (Entity(index), c))
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn query_mut<T: Component + 'static>(&mut self) -> Vec<(Entity, &mut T)> {
        let type_name = std::any::type_name::<T>();
        if let Some(components) = self.components.get_mut(type_name) {
            components
                .iter_mut()
                .enumerate()
                .filter_map(|(index, component)| {
                    component
                        .as_mut()
                        .and_then(|c| c.downcast_mut::<T>())
                        .map(|c| (Entity(index), c))
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_single<T: Component + 'static>(&self) -> Option<&T> {
        let type_name = std::any::type_name::<T>();
        let components = self.components.get(type_name)?;
        if components.len() != 1 {
            None
        } else {
            components
                .iter()
                .filter_map(|c| c.as_ref()?.downcast_ref::<T>())
                .next()
        }
    }

    pub fn get_single_mut<T: Component + 'static>(&mut self) -> Option<&mut T> {
        let type_name = std::any::type_name::<T>();
        let components = self.components.get_mut(type_name)?;
        if components.len() != 1 {
            None
        } else {
            components
                .iter_mut()
                .filter_map(|c| c.as_mut()?.downcast_mut::<T>())
                .next()
        }
    }

    pub fn single<T: Component + 'static>(&self) -> &T {
        self.get_single::<T>()
            .expect("Expected exactly one component")
    }

    pub fn single_mut<T: Component + 'static>(&mut self) -> &mut T {
        self.get_single_mut::<T>()
            .expect("Expected exactly one component")
    }

    // TODO: don't use strings
    pub fn register_schedule(&mut self, name: &'static str) {
        self.schedules.insert(name, Vec::new());
    }

    pub fn add_system(&mut self, schedule_name: &'static str, system: SystemFn) {
        if let Some(systems) = self.schedules.get_mut(schedule_name) {
            systems.push(system);
        }
    }

    pub fn run_system(&mut self, system: SystemFn) {
        system(self);
    }

    pub fn run_schedule(&mut self, schedule_name: &'static str) {
        if let Some(systems) = self.schedules.get(schedule_name).cloned() {
            for system in systems {
                system(self);
            }
        }
    }

    pub fn print_schedules(&self) {
        for (name, systems) in &self.schedules {
            println!("Schedule: {}, Systems: {}", name, systems.len());
        }
    }
}
