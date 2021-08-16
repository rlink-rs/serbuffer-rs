use serbuffer_gen::DataType::*;
use serbuffer_gen::{Codegen, SchemaBuilder};

fn main() {
    Codegen::out_dir("buffer_gen")
        .schema(
            SchemaBuilder::new("GenDemo0")
                .field("timestamp", U64)
                .field("index", U8)
                .field("group", STRING)
                .field("service", I32)
                .field("count", I64)
                .field("bin", BINARY)
                .set_serde_derive(),
        )
        .schema(
            SchemaBuilder::new("GenDemo1")
                .field("timestamp", U64)
                .field("index", U8)
                .field("group", STRING)
                .field("service", I32)
                .field("count", I64)
                .field("bin", BINARY)
                .set_serde_derive(),
        )
        .gen()
        .expect("buffer gen error");
}
