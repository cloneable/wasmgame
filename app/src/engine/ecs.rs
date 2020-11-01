// This tiny ECS is inspired by amethyst/specs + shred.
// I like their API so much, that I wanted to try to replicate
// the inner workings.
use ::std::{
    any::{Any, TypeId},
    boxed::Box,
    cell::{RefCell, RefMut},
    clone::Clone,
    cmp::Ord,
    collections::{btree_map, BTreeMap},
    default::Default,
    iter::Iterator,
    marker::PhantomData,
    option::{Option, Option::None, Option::Some},
    vec::Vec,
};

#[derive(Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct Entity(u32);

#[derive(Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Debug)]
struct ComponentId(TypeId);

impl ComponentId {
    fn of<C: Component>() -> Self {
        ComponentId(TypeId::of::<C>())
    }
}

pub trait Component: Any {}

pub trait System<'a> {
    type Args: Selector<'a>;
    fn exec(&mut self, c: Self::Args);
}

pub struct World {
    components: BTreeMap<ComponentId, RefCell<Box<dyn Any>>>,
    globals: BTreeMap<ComponentId, RefCell<Box<dyn Any>>>,
    entities: u32,
}

impl World {
    pub fn new() -> Self {
        World {
            components: BTreeMap::new(),
            globals: BTreeMap::new(),
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
        let entry = self.components.entry(ComponentId::of::<C>()).or_insert(
            RefCell::new(Box::new(BTreeComponentMap::<C>::default())),
        );
        entry
            .borrow_mut()
            .downcast_mut::<BTreeComponentMap<C>>()
            .unwrap()
            .map
            .insert(entity, component);
    }

    pub fn add_global<C: Component>(&mut self, component: C) {
        self.globals
            .insert(ComponentId::of::<C>(), RefCell::new(Box::new(component)));
    }
}

trait ComponentMap<'a, C: Component>: Any {
    fn iter(&'a self) -> ComponentIter<'a, C>;
    fn iter_mut(&'a mut self) -> ComponentIterMut<'a, C>;
    fn entity_iter(&'a self) -> EntityComponentIter<'a, C>;
    fn entity_iter_mut(&'a mut self) -> EntityComponentIterMut<'a, C>;

    fn get(&'a self, entity: Entity) -> Option<&'a C>;
    fn get_mut(&'a mut self, entity: Entity) -> Option<&'a mut C>;
}

struct BTreeComponentMap<C: Component> {
    map: BTreeMap<Entity, C>,
}

impl<C: Component> Default for BTreeComponentMap<C> {
    fn default() -> Self {
        BTreeComponentMap {
            map: BTreeMap::new(),
        }
    }
}

impl<'b, 'a: 'b, C: Component> ComponentMap<'b, C> for BTreeComponentMap<C> {
    fn iter(&'b self) -> ComponentIter<'b, C> {
        ComponentIter::wrap(self.map.iter())
    }
    fn iter_mut(&'b mut self) -> ComponentIterMut<'b, C> {
        ComponentIterMut::wrap(self.map.iter_mut())
    }
    fn entity_iter(&'b self) -> EntityComponentIter<'b, C> {
        EntityComponentIter::wrap(self.map.iter())
    }
    fn entity_iter_mut(&'b mut self) -> EntityComponentIterMut<'b, C> {
        EntityComponentIterMut::wrap(self.map.iter_mut())
    }

    fn get(&'b self, entity: Entity) -> Option<&'b C> {
        self.map.get(&entity)
    }
    fn get_mut(&'b mut self, entity: Entity) -> Option<&'b mut C> {
        self.map.get_mut(&entity)
    }
}

struct ComponentIter<'a, C: Component> {
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

struct ComponentIterMut<'a, C: Component> {
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

struct EntityComponentIter<'a, C: Component> {
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

struct EntityComponentIterMut<'a, C: Component> {
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
    type Component: Component;
    fn build(world: &'a World) -> Self;
}

macro_rules! tuple_selector_impl {
    ( $( $s:ident),* ) => {
        impl<'a, $($s: Component),*> Component for ($($s,)*) {}

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
    _ecm: &'a RefCell<Box<dyn Any>>,
    ecm: RefMut<'a, Box<dyn Any>>,
    _c: PhantomData<C>,
}

impl<'b, 'a: 'b, C: Component> PerEntity<'a, C> {
    fn new(world: &'a World) -> Self {
        let _ecm = world.components.get(&ComponentId::of::<C>()).unwrap();
        let ecm = _ecm.borrow_mut();
        PerEntity {
            _ecm,
            ecm,
            _c: PhantomData,
        }
    }

    pub fn stream(&'b mut self) -> impl Iterator<Item = &'b C> {
        self.ecm
            .downcast_mut::<BTreeComponentMap<C>>()
            .unwrap()
            .iter()
    }

    pub fn stream_mut(&'b mut self) -> impl Iterator<Item = &'b mut C> {
        self.ecm
            .downcast_mut::<BTreeComponentMap<C>>()
            .unwrap()
            .iter_mut()
    }
}

impl<'a, C: Component> Selector<'a> for PerEntity<'a, C> {
    type Component = C;
    fn build(world: &'a World) -> Self {
        PerEntity::new(world)
    }
}

pub struct Global<'a, C: Component> {
    _c: &'a RefCell<Box<dyn Any>>,
    c: RefMut<'a, Box<dyn Any>>,
    _x: PhantomData<&'a C>,
}

impl<'b, 'a: 'b, C: Component> Global<'a, C> {
    fn new(world: &'a World) -> Self {
        let _c = world.globals.get(&ComponentId::of::<C>()).unwrap();
        let c = _c.borrow_mut();
        Global {
            _c,
            c,
            _x: PhantomData,
        }
    }

    pub fn get(&'b self) -> &'b C {
        self.c.downcast_ref().unwrap()
    }

    pub fn get_mut(&'b mut self) -> &'b mut C {
        self.c.downcast_mut().unwrap()
    }
}

impl<'a, C: Component> Selector<'a> for Global<'a, C> {
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
    impl Component for TestComponentA {}

    #[derive(PartialEq, Eq, Debug)]
    struct TestComponentB(usize);
    impl Component for TestComponentB {}

    #[derive(PartialEq, Eq, Debug)]
    struct GlobalTestComponent(usize);
    impl Component for GlobalTestComponent {}

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
        world.add_global(GlobalTestComponent(3));
        let e1 = world.add_entity();
        world.add_component(e1, TestComponentA(10));
        world.add_component(e1, TestComponentB(100));
        let e2 = world.add_entity();
        world.add_component(e2, TestComponentA(20));
        world.add_component(e2, TestComponentB(200));

        assert_eq!(world.entities, 2);
        assert_eq!(world.components.len(), 2);
        assert_eq!(world.globals.len(), 1);

        let mut r = Runner::new();
        r.register_system(TestSystemA);
        r.register_system(TestSystemB);
        r.exec(&world);

        let ecm_a = world
            .components
            .get(&ComponentId::of::<TestComponentA>())
            .unwrap()
            .borrow();
        let comp_a1 = ecm_a
            .downcast_ref::<BTreeComponentMap<TestComponentA>>()
            .unwrap()
            .get(e1)
            .unwrap();
        let comp_a2 = ecm_a
            .downcast_ref::<BTreeComponentMap<TestComponentA>>()
            .unwrap()
            .get(e2)
            .unwrap();

        let ecm_b = world
            .components
            .get(&ComponentId::of::<TestComponentB>())
            .unwrap()
            .borrow();
        let comp_b1 = ecm_b
            .downcast_ref::<BTreeComponentMap<TestComponentB>>()
            .unwrap()
            .get(e1)
            .unwrap();
        let comp_b2 = ecm_b
            .downcast_ref::<BTreeComponentMap<TestComponentB>>()
            .unwrap()
            .get(e2)
            .unwrap();

        assert_eq!(comp_a1, &TestComponentA(11));
        assert_eq!(comp_a2, &TestComponentA(21));
        assert_eq!(comp_b1, &TestComponentB(103));
        assert_eq!(comp_b2, &TestComponentB(203));
    }
}
