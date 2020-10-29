// This tiny ECS is inspired by amethyst/specs + shred.
// I like their API so much, that I wanted to try to replicate
// the inner workings.
use ::std::{
    any::{Any, TypeId},
    boxed::Box,
    cell::{RefCell, RefMut},
    clone::Clone,
    cmp::Ord,
    collections::{btree_map::IterMut, BTreeMap},
    iter::Iterator,
    marker::PhantomData,
    option::{Option, Option::None, Option::Some},
    vec::Vec,
};

#[derive(Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct Entity(u32);

#[derive(Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct ComponentId(TypeId);

pub trait Component: Any {}

pub trait System<'a> {
    type Args: Selector<'a>;
    fn exec(&mut self, c: Self::Args);
}

pub struct World {
    components: BTreeMap<TypeId, RefCell<EntityComponentMap>>,
    entities: u32,
}

impl<'a> World {
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
            .entry(TypeId::of::<C>())
            .or_insert(RefCell::new(EntityComponentMap::new()));
        entry.borrow_mut().map.insert(entity, Box::new(component));
    }
}

struct EntityComponentMap {
    map: BTreeMap<Entity, Box<dyn Any + 'static>>,
}

impl EntityComponentMap {
    fn new() -> Self {
        EntityComponentMap {
            map: BTreeMap::new(),
        }
    }

    pub fn iter_mut<C: Component>(
        &mut self,
    ) -> ComponentIter<C, IterMut<Entity, Box<dyn Any + 'static>>> {
        ComponentIter {
            iter: self.map.iter_mut(),
            _c: PhantomData,
        }
    }
}

struct ComponentIter<'a, C, I>
where
    C: Component + 'a,
    I: Iterator<Item = (&'a Entity, &'a mut Box<dyn Any + 'static>)> + 'a,
{
    iter: I,
    _c: PhantomData<C>,
}

impl<'a, C, I> Iterator for ComponentIter<'a, C, I>
where
    C: Component + 'a,
    I: Iterator<Item = (&'a Entity, &'a mut Box<dyn Any + 'static>)> + 'a,
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
    fn build(db: &'a World) -> Self;
}

impl<'a, S1> Selector<'a> for (S1,)
where
    S1: Selector<'a>,
{
    fn build(db: &'a World) -> Self {
        (S1::build(db),)
    }
}

impl<'a, S1, S2> Selector<'a> for (S1, S2)
where
    S1: Selector<'a>,
    S2: Selector<'a>,
{
    fn build(db: &'a World) -> Self {
        (S1::build(db), S2::build(db))
    }
}

pub struct Provider<'a, C: Component + 'a> {
    _ecm: &'a RefCell<EntityComponentMap>,
    ecm: RefMut<'a, EntityComponentMap>,
    _c: PhantomData<&'a C>,
}

impl<'a, C: Component + 'a> Provider<'a, C> {
    fn new(db: &'a World) -> Self {
        let _ecm = db.components.get(&TypeId::of::<C>()).unwrap();
        let ecm = _ecm.borrow_mut();
        Provider {
            _ecm,
            ecm,
            _c: PhantomData,
        }
    }
}

impl<'a, 'b: 'a, C: Component> Provider<'b, C> {
    pub fn stream_mut(&'a mut self) -> impl Iterator<Item = &'a mut C> {
        self.ecm.iter_mut()
    }
}

impl<'a, C: Component> Selector<'a> for Provider<'a, C> {
    fn build(db: &'a World) -> Self {
        Provider::new(db)
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

    pub fn exec(&mut self, db: &'a World) {
        for system in self.systems.iter_mut() {
            system.exec(db);
        }
    }
}

trait SystemAdaptor<'a> {
    fn exec(&mut self, db: &'a World);
}

impl<'a, S: System<'a>> SystemAdaptor<'a> for S {
    fn exec(&mut self, db: &'a World) {
        S::exec(self, <S::Args as Selector<'a>>::build(db))
    }
}

#[cfg(test)]
pub mod tests {
    use ::std::{assert_eq, panic};
    use ::wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    #[derive(PartialEq, Eq, Debug)]
    struct TestComponentA(usize);
    #[derive(PartialEq, Eq, Debug)]
    struct TestComponentB(usize);

    impl Component for TestComponentA {}
    impl Component for TestComponentB {}

    struct TestSystemA;

    impl<'a> System<'a> for TestSystemA {
        type Args =
            (Provider<'a, TestComponentA>, Provider<'a, TestComponentB>);
        fn exec(&mut self, (mut comp_a, mut _comp_b): Self::Args) {
            for c in comp_a.stream_mut() {
                c.0 += 1
            }
        }
    }

    struct TestSystemB;

    impl<'a> System<'a> for TestSystemB {
        type Args =
            (Provider<'a, TestComponentA>, Provider<'a, TestComponentB>);
        fn exec(&mut self, (mut _comp_a, mut comp_b): Self::Args) {
            for c in comp_b.stream_mut() {
                c.0 += 1
            }
        }
    }

    #[wasm_bindgen_test]
    fn test_lookup() {
        let mut world = World::new();
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
            .get(&TypeId::of::<TestComponentA>())
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
            .get(&TypeId::of::<TestComponentB>())
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
        assert_eq!(comp_b1, &TestComponentB(101));
        assert_eq!(comp_b2, &TestComponentB(201));
    }
}
