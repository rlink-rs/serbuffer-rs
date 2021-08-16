use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::{env, fs};

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub struct SchemaBuilder {
    schema: String,
    schema_snake: String,
    fields: Vec<Filed>,
    serde_derive: bool,
}

impl SchemaBuilder {
    pub fn new(schema: &str) -> Self {
        let schema_snake = to_snake(schema);
        SchemaBuilder {
            schema: schema.to_string(),
            schema_snake,
            fields: vec![],
            serde_derive: false,
        }
    }

    pub fn field(mut self, name: &str, data_type: DataType) -> Self {
        self.fields.push(Filed {
            name: name.to_string(),
            data_type,
        });

        self
    }

    pub fn set_serde_derive(mut self) -> Self {
        self.serde_derive = true;
        self
    }

    pub(crate) fn build_script(&self) -> String {
        let use_script = self.build_use();
        let field_indies = self.build_field_index();
        let data_type = self.build_data_type();
        let field_name = self.build_field_name();
        let field_metadata = self.build_field_metadata();
        let field_reader = self.build_field_reader();
        let field_writer = self.build_field_writer();
        let entity = self.build_entity();

        format!(
            r#"#![allow(unknown_lints)]
#![allow(clippy::all)]

#![allow(unused_attributes)]
#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unused_imports)]
#![allow(unused_results)]
//! Generated file by schema {}, version {}

{}
{}
{}
{}
{}
{}
{}
{}
"#,
            self.schema,
            VERSION,
            use_script.trim_end(),
            field_indies.trim_end(),
            data_type.trim_end(),
            field_name.trim_end(),
            field_metadata.trim_end(),
            field_reader.trim_end(),
            field_writer.trim_end(),
            entity.trim_end(),
        )
    }

    fn build_use(&self) -> String {
        "use serbuffer::{types, BufferReader, BufferWriter, Buffer, FieldMetadata};\n".to_string()
    }

    fn build_field_index(&self) -> String {
        let mut indies = Vec::new();
        for index in 0..self.fields.len() {
            let field_index = format!(
                "    pub const {}: usize = {};",
                &self.fields[index].name, index
            );
            indies.push(field_index);
        }

        format!(
            r#"
pub mod index {{
{}
}}
        "#,
            indies.join("\n")
        )
    }

    fn build_data_type(&self) -> String {
        let mut field_script = "".to_string();
        for index in 0..self.fields.len() {
            let field = self.fields.get(index).unwrap();
            let dt = format!("{}", field.data_type);
            let data_type = format!(
                r#"    // {}: {}
    types::{},
"#,
                index,
                field.name,
                dt.to_uppercase()
            );
            field_script = format!("{}{}", field_script, data_type);
        }

        let script = format!(
            r#"
pub const FIELD_TYPE: [u8; {}] = [
{}
];"#,
            self.fields.len(),
            field_script.trim_end(),
        );

        script
    }

    fn build_field_name(&self) -> String {
        let mut field_script = "".to_string();
        for index in 0..self.fields.len() {
            let field = self.fields.get(index).unwrap();
            let data_type = format!(
                r#"    // {}: {}
    "{}",
"#,
                index, field.name, field.name,
            );
            field_script = format!("{}{}", field_script, data_type);
        }

        let script = format!(
            r#"
pub const FIELD_NAME: [&'static str; {}] = [
{}
];"#,
            self.fields.len(),
            field_script.trim_end(),
        );

        script
    }

    fn build_field_metadata(&self) -> String {
        format!(
            r#"
pub const FIELD_METADATA: FieldMetadata<{}> = FieldMetadata::new(&FIELD_TYPE, &FIELD_NAME);
"#,
            self.fields.len(),
        )
    }

    fn build_field_reader(&self) -> String {
        let mut field_read_method = "".to_string();
        for index in 0..self.fields.len() {
            let field = self.fields.get(index).unwrap();
            let method_script = match field.data_type {
                DataType::BINARY => format!(
                    r#"
    pub fn get_{}(&mut self) -> Result<&[u8], std::io::Error> {{
        self.reader.get_binary({})
    }}
"#,
                    field.name, index
                ),
                DataType::STRING => format!(
                    r#"
    pub fn get_{}(&mut self) -> Result<&str, std::io::Error> {{
        self.reader.get_str({})
    }}
"#,
                    field.name, index
                ),
                _ => format!(
                    r#"
    pub fn get_{}(&mut self) -> Result<{}, std::io::Error> {{
        self.reader.get_{}({})
    }}
"#,
                    field.name, field.data_type, field.data_type, index
                ),
            };

            field_read_method = format!("{}{}", field_read_method, method_script);
        }

        format!(
            r#"
pub struct FieldReader<'a> {{
    reader: BufferReader<'a, 'static>,
}}

impl<'a> FieldReader<'a> {{
    pub fn new(b: &'a mut Buffer) -> Self {{
        let reader = b.as_reader(&FIELD_TYPE);
        FieldReader {{ reader }}
    }}
{}
}}"#,
            field_read_method.trim_end()
        )
    }

    fn build_field_writer(&self) -> String {
        let mut field_read_method = "".to_string();
        for index in 0..self.fields.len() {
            let field = self.fields.get(index).unwrap();
            let method_script = match field.data_type {
                DataType::BINARY => format!(
                    r#"
    pub fn set_{}(&mut self, {}: &[u8]) -> Result<(), std::io::Error> {{
        if self.writer_pos == {} {{
            self.writer_pos += 1;
            self.writer.set_binary({})
        }} else {{
            Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "`{}` must be set sequentially"))
        }}
    }}
"#,
                    field.name, field.name, index, field.name, field.name,
                ),
                DataType::STRING => format!(
                    r#"
    pub fn set_{}(&mut self, {}: &str) -> Result<(), std::io::Error> {{
        if self.writer_pos == {} {{
            self.writer_pos += 1;
            self.writer.set_str({})
        }} else {{
            Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "`{}` must be set sequentially"))
        }}
    }}
"#,
                    field.name, field.name, index, field.name, field.name,
                ),
                _ => format!(
                    r#"
    pub fn set_{}(&mut self, {}: {}) -> Result<(), std::io::Error> {{
        if self.writer_pos == {} {{
            self.writer_pos += 1;
            self.writer.set_{}({})
        }} else {{
            Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "`{}` must be set sequentially"))
        }}
    }}
"#,
                    field.name,
                    field.name,
                    field.data_type,
                    index,
                    field.data_type,
                    field.name,
                    field.name,
                ),
            };

            field_read_method = format!("{}{}", field_read_method, method_script);
        }

        format!(
            r#"
pub struct FieldWriter<'a> {{
    writer: BufferWriter<'a, 'static>,
    writer_pos: usize,
}}

impl<'a> FieldWriter<'a> {{
    pub fn new(b: &'a mut Buffer) -> Self {{
        let writer = b.as_writer(&FIELD_TYPE);
        FieldWriter {{
            writer,
            writer_pos: 0,
        }}
    }}
{}
}}"#,
            field_read_method.trim_end()
        )
    }

    fn build_entity(&self) -> String {
        let mut ref_type = false;
        let mut fields = "".to_string();
        let mut writers = "".to_string();
        let mut readers = "".to_string();

        for index in 0..self.fields.len() {
            let field = self.fields.get(index).unwrap();

            match field.data_type {
                DataType::BINARY => {
                    ref_type = true;
                    fields = format!("{}\n    pub {}: &'a [u8],", fields, field.name);
                    writers = format!(
                        "{}\n        writer.set_binary(self.{})?;",
                        writers, field.name
                    );
                    readers = format!(
                        "{}\n            {}: reader.get_binary({})?,",
                        readers, field.name, index
                    );
                }
                DataType::STRING => {
                    ref_type = true;
                    fields = format!("{}\n    pub {}: &'a str,", fields, field.name);
                    writers = format!("{}\n        writer.set_str(self.{})?;", writers, field.name);
                    readers = format!(
                        "{}\n            {}: reader.get_str({})?,",
                        readers, field.name, index
                    );
                }
                _ => {
                    fields = format!("{}\n    pub {}: {},", fields, field.name, field.data_type);
                    writers = format!(
                        "{}\n        writer.set_{}(self.{})?;",
                        writers, field.data_type, field.name
                    );
                    readers = format!(
                        "{}\n            {}: reader.get_{}({})?,",
                        readers, field.name, field.data_type, index
                    );
                }
            };
        }

        let ref_type = if ref_type { "<'a>" } else { "" };
        let serde_derive = if self.serde_derive {
            ", Serialize, Deserialize"
        } else {
            ""
        };

        format!(
            r#"
#[derive(Clone, Debug{})]
pub struct Entity{} {{
    {}
}}

impl{} Entity{} {{
    pub fn to_buffer(&self, b: &mut Buffer) -> Result<(), std::io::Error> {{
        let mut writer = b.as_writer(&FIELD_TYPE);
        
        {}

        Ok(())
    }}
    
    pub fn parse(b: &'a mut Buffer) -> Result<Self, std::io::Error> {{
        let reader = b.as_reader(&FIELD_TYPE);

        let entity = Entity {{
            {}
        }};

        Ok(entity)
    }}
}}
            "#,
            serde_derive,
            ref_type,
            fields.trim_start(),
            ref_type,
            ref_type,
            writers.trim_start(),
            readers.trim_start()
        )
    }
}

/// code gen
/// struct Demo {
///   timestamp: u64
/// }
#[derive(Default)]
pub struct Codegen {
    /// --lang_out= param
    generated_dir: PathBuf,
    schemas: Vec<SchemaBuilder>,
}

impl Codegen {
    pub fn new(generated_dir: &str) -> Self {
        if Path::new(generated_dir).exists() {
            fs::remove_dir_all(generated_dir).unwrap();
        }
        fs::create_dir(generated_dir).unwrap();

        Codegen {
            generated_dir: PathBuf::from(generated_dir),
            schemas: vec![],
        }
    }

    pub fn out_dir(path: &str) -> Self {
        let out_dir = env::var("OUT_DIR").unwrap();
        let generated_dir = Path::new(out_dir.as_str()).join(path);

        Self::new(generated_dir.as_path().to_str().unwrap())
    }

    pub fn schema(mut self, schema_builder: SchemaBuilder) -> Self {
        self.schemas.push(schema_builder);
        self
    }

    pub fn gen(&self) -> std::io::Result<()> {
        for schema_builder in &self.schemas {
            let script = schema_builder.build_script();

            let file_name = format!("{}.rs", schema_builder.schema_snake);
            let file_path = self.generated_dir.join(file_name.as_str());
            let mut file_writer = File::create(&file_path)?;
            file_writer.write_all(script.as_bytes())?;
            file_writer.flush()?;
        }

        {
            let mods_str: Vec<String> = self
                .schemas
                .iter()
                .map(|x| format!("pub mod {};", x.schema_snake))
                .collect();
            let script = mods_str.join("\n");

            let file_name = "mod.rs";
            let file_path = self.generated_dir.join(file_name);
            let mut file_writer = File::create(&file_path)?;
            file_writer.write_all(script.as_bytes())?;
            file_writer.flush()?;
        }

        Ok(())
    }
}

fn to_snake(s: &str) -> String {
    let mut v = Vec::new();
    for c in s.chars() {
        if c.is_uppercase() {
            v.push('_');
            v.push(c.to_ascii_lowercase());
        } else {
            v.push(c);
        }
    }

    let snake = String::from_iter(v.iter());
    if snake.starts_with("_") {
        let ss = snake.as_str();
        ss[1..snake.len()].to_string()
    } else {
        snake
    }
}

pub enum DataType {
    BOOL,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    F32,
    F64,
    BINARY,
    STRING,
}

impl Display for DataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::BOOL => write!(f, "{}", "BOOL".to_lowercase()),
            DataType::U8 => write!(f, "{}", "U8".to_lowercase()),
            DataType::I8 => write!(f, "{}", "I8".to_lowercase()),
            DataType::U16 => write!(f, "{}", "U16".to_lowercase()),
            DataType::I16 => write!(f, "{}", "I16".to_lowercase()),
            DataType::U32 => write!(f, "{}", "U32".to_lowercase()),
            DataType::I32 => write!(f, "{}", "I32".to_lowercase()),
            DataType::U64 => write!(f, "{}", "U64".to_lowercase()),
            DataType::I64 => write!(f, "{}", "I64".to_lowercase()),
            DataType::F32 => write!(f, "{}", "F32".to_lowercase()),
            DataType::F64 => write!(f, "{}", "F64".to_lowercase()),
            DataType::BINARY => write!(f, "{}", "BINARY".to_lowercase()),
            DataType::STRING => write!(f, "{}", "STRING".to_lowercase()),
        }
    }
}

impl TryFrom<&str> for DataType {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_uppercase().as_str() {
            "BOOL" => Ok(DataType::BOOL),
            "U8" => Ok(DataType::U8),
            "I8" => Ok(DataType::I8),
            "U16" => Ok(DataType::U16),
            "I16" => Ok(DataType::I16),
            "U32" => Ok(DataType::U32),
            "I32" => Ok(DataType::I32),
            "U64" => Ok(DataType::U64),
            "I64" => Ok(DataType::I64),
            "F32" => Ok(DataType::F32),
            "F64" => Ok(DataType::F64),
            "BINARY" => Ok(DataType::BINARY),
            "STRING" => Ok(DataType::STRING),
            _ => Err("unknown"),
        }
    }
}

struct Filed {
    name: String,
    data_type: DataType,
}

#[cfg(test)]
mod tests {
    use crate::{DataType, SchemaBuilder};

    #[test]
    pub fn code_gen_basic_type_test() {
        let script = SchemaBuilder::new("DemoSchema")
            .field("timestamp", DataType::U64)
            .field("a", DataType::BOOL)
            .field("a1", DataType::U8)
            .build_script();

        println!("-- Start ---");
        println!("{}", script);
        println!("-- End ---");
    }

    #[test]
    pub fn code_gen_ref_type_test() {
        let script = SchemaBuilder::new("DemoSchema")
            .field("timestamp", DataType::U64)
            .field("application_name", DataType::STRING)
            .field("agent_id", DataType::STRING)
            .field("a", DataType::BOOL)
            .field("a1", DataType::U8)
            .field("a2", DataType::BINARY)
            .build_script();

        println!("-- Start ---");
        println!("{}", script);
        println!("-- End ---");
    }
}
