pub mod actor;
pub use broadway_macro;
pub mod venue;
pub mod backstage;

use crate::backstage::Lease;
use crate::backstage::Backstage;
use crate::venue::Venue;
use crate::actor::Role;

use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, Ordering};

pub struct BroadwayContext<B: Backstage>{
    backstage: B,
    venue: Venue<B>,
}

impl<B: Backstage> BroadwayContext<B>{
    async fn get_actor<T: Role + ?Sized>(&self, key: T::Key) -> Lease<T>{
        self.backstage.get_actor::<T>(key).await
    }
}

unsafe impl<B: Backstage> Send for BroadwayContext<B>{}
unsafe impl<B: Backstage> Sync for BroadwayContext<B>{}