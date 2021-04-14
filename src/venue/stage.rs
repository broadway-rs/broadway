use dashmap::DashMap;
use std::sync::Arc;
use crate::actor::{Role, ActorChannel, ActorInstance};
use crate::BroadwayContext;

/// This might get removed, but the idea with this is that
/// it's the services view of the actor, since the service
/// doesn't need to know much about the internals of an actor
/// and an actor doesn't need to know how it looks to the service
/// except for a few signals.
struct Actor<T: Role + ?Sized>{
    status: ActorStatus,
    comms: ActorChannel<T>,
    instance: Option<ActorInstance<T>>
}

pub enum ActorStatus{
    Starting,
    Running,
    ShuttingDown,
    Off
}

pub struct Stage<T: Role + ?Sized>{
    actors: DashMap<T::Key, Actor<T>>,
    ctx: Arc<BroadwayContext>,
}

impl<T: Role + ?Sized + 'static> Stage<T>{
    pub fn new(ctx: Arc<BroadwayContext>) -> Self{
        Self{
            actors: DashMap::new(),
            ctx,
        }
    }

    pub fn get_actor(&self, key: T::Key) -> ActorChannel<T>{
        self.actors
            .entry(key)
            .or_insert_with(||{
                let (channel, instance) = ActorInstance::<T>::start();
                Actor{
                    status: ActorStatus::Starting,
                    comms: channel,
                    instance: Some(instance),
                }
            })
            .downgrade()
            .comms
            .clone()
    }
}