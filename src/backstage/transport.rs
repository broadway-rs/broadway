use crate::data::LocationBlob;

pub trait Transport: Send + Sync{
    fn get_local(&self) -> LocationBlob;
}

pub trait Location{
    
}