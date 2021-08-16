#[macro_use]
extern crate serde_derive;

pub mod buffer_gen {
    include!(concat!(env!("OUT_DIR"), "/buffer_gen/mod.rs"));
}

pub use buffer_gen::gen_demo0::FIELD_TYPE;
