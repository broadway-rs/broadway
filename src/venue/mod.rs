mod stage;

use stage::Stage;
use dashmap::DashMap;
use std::any::{Any, TypeId};
use std::sync::Arc;
use crate::actor::{Role, ActorChannel};
use crate::BroadwayContext;

pub struct Venue{
    stages: DashMap<TypeId, Box<dyn Any>>,
    ctx: Arc<BroadwayContext>,
}

impl Venue{
    pub fn new(ctx: BroadwayContext) -> Self{
        Self{
            stages: DashMap::new(),
            ctx: Arc::new(ctx),
        }
    }

    pub fn get_actor<T>(&self, key: T::Key) -> ActorChannel<T>
        where T: Role + ?Sized + 'static{
        let type_id = TypeId::of::<T>();
        self.stages
            .entry(type_id.clone())
            .or_insert(Box::new(Stage::<T>::new(self.ctx.clone())))
            .downgrade()
            .value()
            .downcast_ref::<Stage<T>>()
            .unwrap()
            .get_actor(key)
    }
}