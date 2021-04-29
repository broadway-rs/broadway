mod stage;

use stage::Stage;
use dashmap::DashMap;
use std::any::{Any, TypeId};
use std::sync::{Weak, Arc};
use crate::actor::{Role, ActorChannel};
use crate::BroadwayContext;

use crate::Backstage;

pub struct Venue<B: Backstage>{
    stages: DashMap<TypeId, Box<dyn Any>>,
    ctx: Weak<BroadwayContext<B>>,
}

impl<B: Backstage + 'static> Venue<B>{
    pub fn new(ctx: Weak<BroadwayContext<B>>) -> Self{
        Self{
            stages: DashMap::new(),
            ctx,
        }
    }

    pub async fn get_actor<T>(&self, key: T::Key) -> ActorChannel<T>
        where T: Role + ?Sized + 'static{
        let type_id = TypeId::of::<T>();
        let value = self.stages
            .entry(type_id.clone())
            .or_insert(Box::new(Stage::<T, B>::new(self.ctx.clone().upgrade().unwrap()).await))
            .downgrade();
        println!("{}", std::any::type_name_of_val(&value));
        value
            .value()
            .downcast_ref::<Box<Stage<T, B>>>()
            .unwrap()
            .get_actor(key)
            .await
    }
}