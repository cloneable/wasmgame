// This tiny ECS is inspired by amethyst/specs + shred.
// I like their API so much, that I wanted to try to replicate
// the inner workings.
use ::std::{
    any::{Any, TypeId},
    boxed::Box,
    cell::{RefCell, RefMut},
    clone::Clone,
    cmp::Ord,
    collections::{btree_map::Iter, btree_map::IterMut, BTreeMap},
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
    components: BTreeMap<ComponentId, RefCell<EntityComponentMap>>,
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
        let entry = self
            .components
            .entry(ComponentId::of::<C>())
            .or_insert(RefCell::new(EntityComponentMap::new()));
        entry.borrow_mut().map.insert(entity, Box::new(component));
    }

    pub fn add_global<C: Component>(&mut self, component: C) {
        self.globals
            .insert(ComponentId::of::<C>(), RefCell::new(Box::new(component)));
    }
}

struct EntityComponentMap {
    map: BTreeMap<Entity, Box<dyn Any>>,
}

impl EntityComponentMap {
    fn new() -> Self {
        EntityComponentMap {
            map: BTreeMap::new(),
        }
    }

    pub fn iter<C: Component>(
        &self,
    ) -> ComponentIter<C, Iter<Entity, Box<dyn Any>>> {
        ComponentIter {
            iter: self.map.iter(),
            _c: PhantomData,
        }
    }

    pub fn iter_mut<C: Component>(
        &mut self,
    ) -> ComponentIterMut<C, IterMut<Entity, Box<dyn Any>>> {
        ComponentIterMut {
            iter: self.map.iter_mut(),
            _c: PhantomData,
        }
    }
}

struct ComponentIter<'a, C, I>
where
    C: Component,
    I: Iterator<Item = (&'a Entity, &'a Box<dyn Any>)>,
{
    iter: I,
    _c: PhantomData<C>,
}

impl<'a, C, I> Iterator for ComponentIter<'a, C, I>
where
    C: Component,
    I: Iterator<Item = (&'a Entity, &'a Box<dyn Any>)>,
{
    type Item = &'a C;
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some((_entity, any)) => Some(any.downcast_ref::<C>().unwrap()),
            None => None,
        }
    }
}

struct ComponentIterMut<'a, C, I>
where
    C: Component,
    I: Iterator<Item = (&'a Entity, &'a mut Box<dyn Any>)>,
{
    iter: I,
    _c: PhantomData<C>,
}

impl<'a, C, I> Iterator for ComponentIterMut<'a, C, I>
where
    C: Component,
    I: Iterator<Item = (&'a Entity, &'a mut Box<dyn Any>)>,
{
    type Item = &'a mut C;
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some((_entity, any)) => Some(any.downcast_mut::<C>().unwrap()),
            None => None,
        }
    }
}

pub trait Selector<'a> {
    fn build(world: &'a World) -> Self;
}

impl<'a, S1> Selector<'a> for (S1,)
where
    S1: Selector<'a>,
{
    fn build(world: &'a World) -> Self {
        (S1::build(world),)
    }
}

impl<'a, S1, S2> Selector<'a> for (S1, S2)
where
    S1: Selector<'a>,
    S2: Selector<'a>,
{
    fn build(world: &'a World) -> Self {
        (S1::build(world), S2::build(world))
    }
}

pub struct PerEntity<'a, C: Component> {
    _ecm: &'a RefCell<EntityComponentMap>,
    ecm: RefMut<'a, EntityComponentMap>,
    _c: PhantomData<&'a C>,
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

    pub fn stream(&'b self) -> impl Iterator<Item = &'b C> {
        self.ecm.iter()
    }

    pub fn stream_mut(&'b mut self) -> impl Iterator<Item = &'b mut C> {
        self.ecm.iter_mut()
    }
}

impl<'a, C: Component> Selector<'a> for PerEntity<'a, C> {
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
            PerEntity<'a, TestComponentB>,
            Global<'a, GlobalTestComponent>,
        );
        fn exec(&mut self, (mut comp_b, glob): Self::Args) {
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

        let mut r = Runner::new();
        r.register_system(TestSystemA);
        r.register_system(TestSystemB);
        r.exec(&world);

        assert_eq!(world.entities, 2);
        let ecm_a = world
            .components
            .get(&ComponentId::of::<TestComponentA>())
            .unwrap()
            .borrow();
        let comp_a1 = ecm_a
            .map
            .get(&e1)
            .unwrap()
            .downcast_ref::<TestComponentA>()
            .unwrap();
        let comp_a2 = ecm_a
            .map
            .get(&e2)
            .unwrap()
            .downcast_ref::<TestComponentA>()
            .unwrap();
        let ecm_b = world
            .components
            .get(&ComponentId::of::<TestComponentB>())
            .unwrap()
            .borrow();
        let comp_b1 = ecm_b
            .map
            .get(&e1)
            .unwrap()
            .downcast_ref::<TestComponentB>()
            .unwrap();
        let comp_b2 = ecm_b
            .map
            .get(&e2)
            .unwrap()
            .downcast_ref::<TestComponentB>()
            .unwrap();
        assert_eq!(comp_a1, &TestComponentA(11));
        assert_eq!(comp_a2, &TestComponentA(21));
        assert_eq!(comp_b1, &TestComponentB(103));
        assert_eq!(comp_b2, &TestComponentB(203));
    }
}
