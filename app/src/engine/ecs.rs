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
    components: BTreeMap<ComponentId, RefCell<BTreeComponentMap>>,
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
            .or_insert(RefCell::new(BTreeComponentMap::default()));
        entry.borrow_mut().map.insert(entity, Box::new(component));
    }

    pub fn add_global<C: Component>(&mut self, component: C) {
        self.globals
            .insert(ComponentId::of::<C>(), RefCell::new(Box::new(component)));
    }
}

trait ComponentMap<'a, C: Component> {
    type Iter: Iterator<Item = &'a C>;
    type IterMut: Iterator<Item = &'a mut C>;
    type EntityIter: Iterator<Item = (Entity, &'a C)>;
    type EntityIterMut: Iterator<Item = (Entity, &'a mut C)>;

    fn iter(&'a self) -> Self::Iter;
    fn iter_mut(&'a mut self) -> Self::IterMut;
    fn entity_iter(&'a self) -> Self::EntityIter;
    fn entity_iter_mut(&'a mut self) -> Self::EntityIterMut;

    fn get(&'a self, entity: Entity) -> Option<&'a C>;
    fn get_mut(&'a mut self, entity: Entity) -> Option<&'a mut C>;
}

struct BTreeComponentMap {
    map: BTreeMap<Entity, Box<dyn Any>>,
}

impl Default for BTreeComponentMap {
    fn default() -> Self {
        BTreeComponentMap {
            map: BTreeMap::new(),
        }
    }
}

impl<'b, 'a: 'b, C: Component> ComponentMap<'b, C> for BTreeComponentMap {
    type Iter = ComponentIter<'b, C, btree_map::Iter<'b, Entity, Box<dyn Any>>>;
    type IterMut =
        ComponentIterMut<'b, C, btree_map::IterMut<'b, Entity, Box<dyn Any>>>;
    type EntityIter =
        EntityComponentIter<'b, C, btree_map::Iter<'b, Entity, Box<dyn Any>>>;
    type EntityIterMut = EntityComponentIterMut<
        'b,
        C,
        btree_map::IterMut<'b, Entity, Box<dyn Any>>,
    >;

    fn iter(&'b self) -> Self::Iter {
        ComponentIter::wrap(self.map.iter())
    }
    fn iter_mut(&'b mut self) -> Self::IterMut {
        ComponentIterMut::wrap(self.map.iter_mut())
    }
    fn entity_iter(&'b self) -> Self::EntityIter {
        EntityComponentIter::wrap(self.map.iter())
    }
    fn entity_iter_mut(&'b mut self) -> Self::EntityIterMut {
        EntityComponentIterMut::wrap(self.map.iter_mut())
    }

    fn get(&'b self, entity: Entity) -> Option<&'b C> {
        match self.map.get(&entity) {
            Some(ref any) => any.downcast_ref::<C>(),
            None => None,
        }
    }
    fn get_mut(&'b mut self, entity: Entity) -> Option<&'b mut C> {
        match self.map.get_mut(&entity) {
            Some(any) => Some(any.downcast_mut::<C>().unwrap()),
            None => None,
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

impl<'a, C, I> ComponentIter<'a, C, I>
where
    C: Component,
    I: Iterator<Item = (&'a Entity, &'a Box<dyn Any>)>,
{
    fn wrap(iter: I) -> Self {
        ComponentIter {
            iter,
            _c: PhantomData,
        }
    }
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

impl<'a, C, I> ComponentIterMut<'a, C, I>
where
    C: Component,
    I: Iterator<Item = (&'a Entity, &'a mut Box<dyn Any>)>,
{
    fn wrap(iter: I) -> Self {
        ComponentIterMut {
            iter,
            _c: PhantomData,
        }
    }
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

struct EntityComponentIter<'a, C, I>
where
    C: Component,
    I: Iterator<Item = (&'a Entity, &'a Box<dyn Any>)>,
{
    iter: I,
    _c: PhantomData<C>,
}

impl<'a, C, I> EntityComponentIter<'a, C, I>
where
    C: Component,
    I: Iterator<Item = (&'a Entity, &'a Box<dyn Any>)>,
{
    fn wrap(iter: I) -> Self {
        EntityComponentIter {
            iter,
            _c: PhantomData,
        }
    }
}

impl<'a, C, I> Iterator for EntityComponentIter<'a, C, I>
where
    C: Component,
    I: Iterator<Item = (&'a Entity, &'a Box<dyn Any>)>,
{
    type Item = (Entity, &'a C);
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some((entity, any)) => {
                Some((*entity, any.downcast_ref::<C>().unwrap()))
            }
            None => None,
        }
    }
}

struct EntityComponentIterMut<'a, C, I>
where
    C: Component,
    I: Iterator<Item = (&'a Entity, &'a mut Box<dyn Any>)>,
{
    iter: I,
    _c: PhantomData<C>,
}

impl<'a, C, I> EntityComponentIterMut<'a, C, I>
where
    C: Component,
    I: Iterator<Item = (&'a Entity, &'a mut Box<dyn Any>)>,
{
    fn wrap(iter: I) -> Self {
        EntityComponentIterMut {
            iter,
            _c: PhantomData,
        }
    }
}

impl<'a, C, I> Iterator for EntityComponentIterMut<'a, C, I>
where
    C: Component,
    I: Iterator<Item = (&'a Entity, &'a mut Box<dyn Any>)>,
{
    type Item = (Entity, &'a mut C);
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some((entity, any)) => {
                Some((*entity, any.downcast_mut::<C>().unwrap()))
            }
            None => None,
        }
    }
}

pub trait Selector<'a> {
    fn build(world: &'a World) -> Self;
}

macro_rules! tuple_selector_impl {
    ( $( $s:ident),* ) => {
        impl<'a, $($s),*> Selector<'a> for ($($s,)*)
        where
            $($s: Selector<'a>,)*
        {
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
    _ecm: &'a RefCell<BTreeComponentMap>,
    ecm: RefMut<'a, BTreeComponentMap>,
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
