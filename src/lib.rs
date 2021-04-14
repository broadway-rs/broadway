pub mod actor;
pub use broadway_macro;
pub mod venue;
pub mod data;
pub mod backstage;

pub struct BroadwayContext{
    backstage: Box<dyn backstage::Backstage>,
}