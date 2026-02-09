// ECS - the naive way

use wgpu::naga::FastHashMap;

#[derive(Clone, Copy)]
pub struct Entity(usize);

impl Entity {
    pub fn insert<T: Component + 'static>(self, component: T, world: &mut World) -> Self {
        world.add_component(self, component);
        self
    }
    pub fn remove<T: Component + 'static>(self, world: &mut World) -> Self {
        world.remove_component::<T>(self);
        self
    }
}

pub trait Component: std::any::Any + std::fmt::Debug {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

impl dyn Component {
    fn downcast_ref<T: std::any::Any>(&self) -> Option<&T> {
        self.as_any().downcast_ref()
    }
    fn downcast_mut<T: std::any::Any>(&mut self) -> Option<&mut T> {
        self.as_any_mut().downcast_mut()
    }
}

type EntityComponents = Option<Box<dyn Component>>;
type SystemFn = fn(&mut World);

#[derive(Default)]
pub struct World {
    components: FastHashMap<&'static str, Vec<EntityComponents>>,
    resources: FastHashMap<&'static str, Box<dyn Component>>,
    schedules: FastHashMap<&'static str, Vec<SystemFn>>,
}

pub struct EntityWorld<'a> {
    world: &'a mut World,
    entity: Entity,
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

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_component<T: Component + 'static>(&mut self) {
        let type_name = std::any::type_name::<T>();
        self.components.insert(type_name, Vec::new());
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
        }
    }

    pub fn remove_component<T: Component + 'static>(&mut self, entity: Entity) {
        let type_name = std::any::type_name::<T>();
        if let Some(components) = self.components.get_mut(type_name) {
            components[entity.0] = None;
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
