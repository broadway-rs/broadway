mod stage;

use stage::Stage;
use dashmap::DashMap;
use std::any::{Any, TypeId};
use crate::actor::{Role, ActorChannel};

pub struct Venue{
    stages: DashMap<TypeId, Box<dyn Any>>
}

impl Venue{
    pub fn new() -> Self{
        Self{
            stages: DashMap::new(),
        }
    }

    pub fn get_actor<T>(&self, key: T::Key) -> ActorChannel<T>
        where T: Role + ?Sized + 'static{
        let type_id = TypeId::of::<T>();
        self.stages
            .entry(type_id.clone())
            .or_insert(Box::new(Stage::<T>::new()))
            .downgrade()
            .value()
            .downcast_ref::<Stage<T>>()
            .unwrap()
            .get_actor(key)
    }
}