pub mod actor;
pub use broadway_macro;
pub mod venue;
pub mod backstage;

use crate::backstage::{Backstage};
use crate::actor::Role;

use std::sync::Arc;

pub struct BroadwayContext<B: Backstage>{
    backstage: Arc<B>,
}

impl<B: Backstage> Clone for BroadwayContext<B>{
    fn clone(&self) -> Self{
        Self{
            backstage: self.backstage.clone(),
        }
    }
}