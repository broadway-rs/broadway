pub mod actor;
use crate::actor::ActorChannel;
use once_cell::sync::OnceCell;
use core::mem::MaybeUninit;
pub use broadway_macro;
pub mod venue;
pub mod backstage;

use crate::backstage::Lease;
use crate::backstage::Backstage;
use crate::backstage::transport::{Transport, Location};
use crate::venue::Venue;
use crate::actor::Role;

use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, Ordering};

pub struct BroadwayContext<B: Backstage>{
    backstage: OnceCell<B>,
    venue: OnceCell<Venue<B>>,
    transport: Transport
}

impl<B: Backstage + 'static> BroadwayContext<B>{
    pub async fn new(location: Location) -> Arc<Self>{
        let ctx = Arc::new(Self{
            backstage: OnceCell::new(),
            venue: OnceCell::new(),
            transport: Transport::new(location)
        });

        let backstage = B::new(ctx.clone()).await;
        if let Err(_) = ctx.backstage.set(backstage) {panic!("Failed to set backstage!")};
        let venue = Venue::new(Arc::downgrade(&ctx));
        if let Err(_) = ctx.venue.set(venue) {panic!("Failed to set venue!")};
        ctx
    }

    pub async fn get_actor<T: Role + ?Sized + 'static>(&self, key: T::Key) -> ActorChannel<T>{
        self.venue.get().unwrap().get_actor::<T>(key).await
    }
}

unsafe impl<B: Backstage> Send for BroadwayContext<B>{}
unsafe impl<B: Backstage> Sync for BroadwayContext<B>{}