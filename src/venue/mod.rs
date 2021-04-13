mod stage;

use stage::Stage;
use dashmap::DashMap;
use std::any::{Any, TypeId};
use crate::actor::{Role, ActorChannel};
use crate::BroadwayContext;
use crate::backstage::Backstage;

pub struct Broadway<B: Backstage + 'static>{
    stages: DashMap<TypeId, Box<dyn Any>>,
    ctx: BroadwayContext<B>,
}

impl<B: Backstage + 'static> Broadway<B>{
    pub fn new(ctx: BroadwayContext<B>) -> Self{
        Self{
            stages: DashMap::new(),
            ctx
        }
    }

    pub fn get_actor<T>(&self, key: T::Key) -> ActorChannel<T>
        where T: Role + ?Sized + 'static{
        let type_id = TypeId::of::<T>();
        self.stages
            .entry(type_id.clone())
            .or_insert(Box::new(Stage::<T, B>::new(self.ctx.clone())))
            .downgrade()
            .value()
            .downcast_ref::<Stage<T, B>>()
            .unwrap()
            .get_actor(key)
    }
}