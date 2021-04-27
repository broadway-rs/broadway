pub mod actor;
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
    backstage: B,
    venue: OnceCell<Venue<B>>,
    transport: Transport
}

impl<B: Backstage + 'static> BroadwayContext<B>{
    pub fn new(backstage: B, location: Location) -> Arc<Self>{
        let ctx = Arc::new(Self{
            backstage,
            venue: OnceCell::new(),
            transport: Transport::new(location)
        });

        let venue = Venue::new(Arc::downgrade(&ctx));
        if let Err(_) = ctx.venue.set(venue) {panic!("Failed to set venue!")};
        ctx
    }

    async fn get_actor<T: Role + ?Sized>(&self, key: T::Key) -> Lease<T>{
        self.backstage.get_actor::<T>(key).await
    }
}

unsafe impl<B: Backstage> Send for BroadwayContext<B>{}
unsafe impl<B: Backstage> Sync for BroadwayContext<B>{}