// This tiny ECS is inspired by amethyst/specs + shred.
// I like their API so much, that I wanted to try to replicate
// the inner workings.
use ::std::{
    any::{Any, TypeId},
    boxed::Box,
    cell::UnsafeCell,
    clone::Clone,
    cmp::Ord,
    collections::BTreeMap,
    default::Default,
    iter::Iterator,
    marker::Sized,
    mem::MaybeUninit,
    option::{Option, Option::None, Option::Some},
    panic, unimplemented,
    vec::Vec,
};

#[derive(Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct Entity(u32);

#[derive(Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct ComponentId(TypeId);

impl ComponentId {
    fn of<C: Component>() -> Self {
        ComponentId(TypeId::of::<C>())
    }
}

pub trait Component: Any + Sized {
    type Container: Container<Self>;

    fn component_id(&self) -> ComponentId {
        ComponentId(self.type_id())
    }
}

pub trait System<'a> {
    type Args: Selector<'a>;
    fn exec(&mut self, c: Self::Args);
}

pub struct World {
    components: BTreeMap<ComponentId, Box<dyn Any + 'static>>,
    entities: u32,
}

impl World {
    pub fn new() -> Self {
        World {
            components: BTreeMap::new(),
            entities: 0,
        }
    }

    pub fn add_entity(&mut self) -> Entity {
        self.entities += 1;
        Entity(self.entities)
    }

    pub fn add_component<C: Component>(
        &mut self, entity: Entity, component: C,
    ) {
        let entry = self
            .components
            .entry(ComponentId::of::<C>())
            .or_insert(Box::new(C::Container::default()));
        entry
            .downcast_mut::<C::Container>()
            .unwrap()
            .insert(entity, component);
    }

    fn get_container<C: Component>(&self) -> &C::Container {
        let any = self.components.get(&ComponentId::of::<C>()).unwrap();
        any.downcast_ref::<C::Container>().unwrap()
    }
}

pub trait Container<C: Component>: Any + Default {
    fn iter<'a>(&self) -> ComponentIter<'a, C>;
    fn iter_mut<'a>(&self) -> ComponentIterMut<'a, C>;
    fn entity_iter<'a>(&self) -> EntityComponentIter<'a, C>;
    fn entity_iter_mut<'a>(&self) -> EntityComponentIterMut<'a, C>;

    fn get<'a>(&self, entity: Entity) -> Option<&'a C>;
    fn get_mut<'a>(&self, entity: Entity) -> Option<&'a mut C>;

    fn insert(&mut self, entity: Entity, component: C);
}

pub struct Singleton<C: Component> {
    // TODO: Add wrapper type similar to RefMut to get some safety back.
    value: UnsafeCell<MaybeUninit<C>>,
}

impl<C: Component> Default for Singleton<C> {
    fn default() -> Self {
        Singleton {
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }
}

impl<C: Component> Container<C> for Singleton<C> {
    fn iter<'a>(&self) -> ComponentIter<'a, C> {
        unimplemented!()
    }
    fn iter_mut<'a>(&self) -> ComponentIterMut<'a, C> {
        unimplemented!()
    }
    fn entity_iter<'a>(&self) -> EntityComponentIter<'a, C> {
        unimplemented!()
    }
    fn entity_iter_mut<'a>(&self) -> EntityComponentIterMut<'a, C> {
        unimplemented!()
    }

    fn get<'a>(&self, _: Entity) -> Option<&'a C> {
        match unsafe { self.value.get().as_ref() } {
            Some(mu) => unsafe { mu.as_ptr().as_ref() },
            None => None,
        }
    }
    fn get_mut<'a>(&self, _: Entity) -> Option<&'a mut C> {
        match unsafe { self.value.get().as_mut() } {
            Some(mu) => unsafe { mu.as_mut_ptr().as_mut() },
            None => None,
        }
    }

    fn insert(&mut self, _: Entity, component: C) {
        unsafe {
            let mu = self.value.get().as_mut().unwrap();
            mu.as_mut_ptr().write(component);
        }
    }
}

pub struct BTreeComponentMap<C: Component> {
    // TODO: Add wrapper type similar to RefMut to get some safety back.
    map: UnsafeCell<BTreeMap<Entity, C>>,
}

impl<'a, C: Component> BTreeComponentMap<C> {
    fn map(&self) -> &'a BTreeMap<Entity, C> {
        unsafe { self.map.get().as_ref().unwrap() }
    }

    fn map_mut(&self) -> &'a mut BTreeMap<Entity, C> {
        unsafe { self.map.get().as_mut().unwrap() }
    }
}

impl<C: Component> Default for BTreeComponentMap<C> {
    fn default() -> Self {
        BTreeComponentMap {
            map: UnsafeCell::new(BTreeMap::new()),
        }
    }
}

impl<C: Component> Container<C> for BTreeComponentMap<C> {
    fn iter<'a>(&self) -> ComponentIter<'a, C> {
        ComponentIter::wrap(self.map().iter())
    }
    fn iter_mut<'a>(&self) -> ComponentIterMut<'a, C> {
        ComponentIterMut::wrap(self.map_mut().iter_mut())
    }
    fn entity_iter<'a>(&self) -> EntityComponentIter<'a, C> {
        EntityComponentIter::wrap(self.map().iter())
    }
    fn entity_iter_mut<'a>(&self) -> EntityComponentIterMut<'a, C> {
        EntityComponentIterMut::wrap(self.map_mut().iter_mut())
    }

    fn get<'a>(&self, entity: Entity) -> Option<&'a C> {
        self.map().get(&entity)
    }
    fn get_mut<'a>(&self, entity: Entity) -> Option<&'a mut C> {
        self.map_mut().get_mut(&entity)
    }

    fn insert(&mut self, entity: Entity, component: C) {
        self.map_mut().insert(entity, component);
    }
}

pub struct ComponentIter<'a, C: Component> {
    iter: Box<dyn Iterator<Item = (&'a Entity, &'a C)> + 'a>,
}

impl<'a, C: Component> ComponentIter<'a, C> {
    fn wrap(iter: impl Iterator<Item = (&'a Entity, &'a C)> + 'a) -> Self {
        ComponentIter {
            iter: Box::new(iter),
        }
    }
}

impl<'a, C: Component> Iterator for ComponentIter<'a, C> {
    type Item = &'a C;
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some((_entity, component)) => Some(component),
            None => None,
        }
    }
}

pub struct ComponentIterMut<'a, C: Component> {
    iter: Box<dyn Iterator<Item = (&'a Entity, &'a mut C)> + 'a>,
}

impl<'a, C: Component> ComponentIterMut<'a, C> {
    fn wrap(iter: impl Iterator<Item = (&'a Entity, &'a mut C)> + 'a) -> Self {
        ComponentIterMut {
            iter: Box::new(iter),
        }
    }
}

impl<'a, C: Component> Iterator for ComponentIterMut<'a, C> {
    type Item = &'a mut C;
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some((_entity, component)) => Some(component),
            None => None,
        }
    }
}

pub struct EntityComponentIter<'a, C: Component> {
    iter: Box<dyn Iterator<Item = (&'a Entity, &'a C)> + 'a>,
}

impl<'a, C: Component> EntityComponentIter<'a, C> {
    fn wrap(iter: impl Iterator<Item = (&'a Entity, &'a C)> + 'a) -> Self {
        EntityComponentIter {
            iter: Box::new(iter),
        }
    }
}

impl<'a, C: Component> Iterator for EntityComponentIter<'a, C> {
    type Item = (Entity, &'a C);
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some((entity, component)) => Some((*entity, component)),
            None => None,
        }
    }
}

pub struct EntityComponentIterMut<'a, C: Component> {
    iter: Box<dyn Iterator<Item = (&'a Entity, &'a mut C)> + 'a>,
}

impl<'a, C: Component> EntityComponentIterMut<'a, C> {
    fn wrap(iter: impl Iterator<Item = (&'a Entity, &'a mut C)> + 'a) -> Self {
        EntityComponentIterMut {
            iter: Box::new(iter),
        }
    }
}

impl<'a, C: Component> Iterator for EntityComponentIterMut<'a, C> {
    type Item = (Entity, &'a mut C);
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some((entity, component)) => Some((*entity, component)),
            None => None,
        }
    }
}

pub trait Selector<'a> {
    type Component;
    fn build(world: &'a World) -> Self;
}

macro_rules! tuple_selector_impl {
    ( $( $s:ident),* ) => {
        impl<'a, $($s),*> Selector<'a> for ($($s,)*)
        where
            $($s: Selector<'a>,)*
        {
            type Component = ($($s::Component,)*);
            fn build(world: &'a World) -> Self {
                ($($s::build(world),)*)
            }
        }
    }
}

tuple_selector_impl!(S1);
tuple_selector_impl!(S1, S2);
tuple_selector_impl!(S1, S2, S3);

pub struct PerEntity<'a, C: Component> {
    container: &'a C::Container,
}

impl<'a, C: Component> PerEntity<'a, C> {
    fn new(world: &'a World) -> Self {
        PerEntity {
            container: world.get_container::<C>(),
        }
    }

    pub fn stream(&self) -> impl Iterator<Item = &'a C> {
        self.container.iter()
    }

    pub fn stream_mut(&mut self) -> impl Iterator<Item = &'a mut C> {
        self.container.iter_mut()
    }
}

impl<'a, C: Component> Selector<'a> for PerEntity<'a, C> {
    type Component = C;
    fn build(world: &'a World) -> Self {
        PerEntity::new(world)
    }
}

pub struct Global<'a, C>
where
    C: Component<Container = Singleton<C>>,
{
    container: &'a C::Container,
}

impl<'a, C> Global<'a, C>
where
    C: Component<Container = Singleton<C>>,
{
    fn new(world: &'a World) -> Self {
        Global {
            container: world.get_container::<C>(),
        }
    }

    pub fn get(&self) -> &'a C {
        self.container.get(Entity(0)).unwrap()
    }

    pub fn get_mut(&mut self) -> &'a mut C {
        self.container.get_mut(Entity(0)).unwrap()
    }
}

impl<'a, C> Selector<'a> for Global<'a, C>
where
    C: Component<Container = Singleton<C>>,
{
    type Component = C;
    fn build(world: &'a World) -> Self {
        Global::new(world)
    }
}

pub struct Runner<'a> {
    systems: Vec<Box<dyn SystemAdaptor<'a> + 'a>>,
}

impl<'a> Runner<'a> {
    pub fn new() -> Self {
        Runner {
            systems: Vec::new(),
        }
    }

    pub fn register_system<S: System<'a> + 'a>(&mut self, system: S) {
        self.systems.push(Box::new(system));
    }

    pub fn exec(&mut self, world: &'a World) {
        for system in self.systems.iter_mut() {
            system.exec(world);
        }
    }
}

trait SystemAdaptor<'a> {
    fn exec(&mut self, world: &'a World);
}

impl<'a, S: System<'a>> SystemAdaptor<'a> for S {
    fn exec(&mut self, world: &'a World) {
        S::exec(self, <S::Args as Selector<'a>>::build(world))
    }
}

#[cfg(test)]
pub mod tests {
    use ::std::{assert_eq, panic};
    use ::wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    #[derive(PartialEq, Eq, Debug)]
    struct TestComponentA(usize);
    impl Component for TestComponentA {
        type Container = BTreeComponentMap<TestComponentA>;
    }

    #[derive(PartialEq, Eq, Debug)]
    struct TestComponentB(usize);
    impl Component for TestComponentB {
        type Container = BTreeComponentMap<TestComponentB>;
    }

    #[derive(PartialEq, Eq, Debug)]
    struct GlobalTestComponent(usize);
    impl Component for GlobalTestComponent {
        // TODO: Singleton container for globals.
        type Container = Singleton<GlobalTestComponent>;
    }

    struct TestSystemA;

    impl<'a> System<'a> for TestSystemA {
        type Args =
            (PerEntity<'a, TestComponentA>, PerEntity<'a, TestComponentB>);
        fn exec(&mut self, (mut comp_a, mut _comp_b): Self::Args) {
            for c in comp_a.stream_mut() {
                c.0 += 1
            }
        }
    }

    struct TestSystemB;

    impl<'a> System<'a> for TestSystemB {
        type Args = (
            PerEntity<'a, TestComponentA>,
            PerEntity<'a, TestComponentB>,
            Global<'a, GlobalTestComponent>,
        );
        fn exec(&mut self, (mut _comp_a, mut comp_b, glob): Self::Args) {
            for c in comp_b.stream_mut() {
                c.0 += glob.get().0
            }
        }
    }

    #[wasm_bindgen_test]
    fn test_lookup() {
        let mut world = World::new();
        // TODO: provide add_component for global ones.
        world.add_component(Entity(0), GlobalTestComponent(3));
        let e1 = world.add_entity();
        world.add_component(e1, TestComponentA(10));
        world.add_component(e1, TestComponentB(100));
        let e2 = world.add_entity();
        world.add_component(e2, TestComponentA(20));
        world.add_component(e2, TestComponentB(200));

        assert_eq!(world.entities, 2);
        assert_eq!(world.components.len(), 3);

        let mut r = Runner::new();
        r.register_system(TestSystemA);
        r.register_system(TestSystemB);
        r.exec(&world);

        let container_a = world.get_container::<TestComponentA>();
        let comp_a1 = container_a.get(e1).unwrap();
        let comp_a2 = container_a.get(e2).unwrap();

        let container_b = world.get_container::<TestComponentB>();
        let comp_b1 = container_b.get(e1).unwrap();
        let comp_b2 = container_b.get(e2).unwrap();

        assert_eq!(comp_a1, &TestComponentA(11));
        assert_eq!(comp_a2, &TestComponentA(21));
        assert_eq!(comp_b1, &TestComponentB(103));
        assert_eq!(comp_b2, &TestComponentB(203));
    }
}
