use serbuffer_gen::Codegen;
use serbuffer_gen::DataType::*;

fn main() {
    Codegen::new("src/buffer_gen", "GenDemo")
        .field("timestamp", U64)
        .field("index", U8)
        .field("group", STRING)
        .field("service", I32)
        .field("count", I64)
        .gen()
        .expect("buffer gen error");
}
