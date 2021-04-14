use dashmap::DashMap;
use std::sync::Arc;
use crate::actor::{Role, ActorChannel, ActorInstance, remote::RemoteActor};
use crate::BroadwayContext;
use crate::data::*;

/// This might get removed, but the idea with this is that
/// it's the services view of the actor, since the service
/// doesn't need to know much about the internals of an actor
/// and an actor doesn't need to know how it looks to the service
/// except for a few signals.
struct ActorManager<T: Role + ?Sized>{
    status: ActorStatus,
    comms: ActorChannel<T>,
    instance: Option<Actor<T>>
}

enum Actor<T: Role + ?Sized>{
    Local(ActorInstance<T>),
    Remote(RemoteActor<T>)
}

pub enum ActorStatus{
    Starting,
    Running,
    ShuttingDown,
    Off
}

pub struct Stage<T: Role + ?Sized>{
    actors: DashMap<T::Key, ActorManager<T>>,
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
        // Check if we already have it
        if let dashmap::mapref::entry::Entry::Occupied(oc) = self.actors
            .entry(key){
                return oc.get().comms.clone()
            };

        let key = KeyBlob::new::<T>(key);
        match self.ctx.backstage.get_actor()
    }
}