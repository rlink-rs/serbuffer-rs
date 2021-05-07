use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};

/// code gen
/// struct Demo {
///   timestamp: u64
/// }
#[derive(Default)]
pub struct Codegen {
    /// --lang_out= param
    out_dir: PathBuf,
    schema: String,
    schema_snake: String,
    fields: Vec<Filed>,
}

impl Codegen {
    pub fn new(out_dir: impl AsRef<Path>, schema: &str) -> Self {
        let schema_snake = to_snake(schema);
        Codegen {
            out_dir: out_dir.as_ref().to_owned(),
            schema: schema.to_string(),
            schema_snake,
            fields: Vec::new(),
        }
    }

    pub fn field(&mut self, name: &str, data_type: DataType) -> &mut Self {
        self.fields.push(Filed {
            name: name.to_string(),
            data_type,
        });

        self
    }

    pub fn gen(&self) -> std::io::Result<()> {
        let script = self.build_script();

        let file_name = format!("{}.rs", self.schema_snake);
        let file_path = self.out_dir.join(file_name.as_str());
        let mut file_writer = File::create(&file_path)?;
        file_writer.write_all(script.as_bytes())?;
        file_writer.flush()?;

        Ok(())
    }

    pub(crate) fn build_script(&self) -> String {
        let use_script = self.build_use();
        let field_indies = self.build_field_index();
        let data_type = self.build_data_type();
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
//! Generated file by schema {}

{}
{}
{}
{}
{}
{}
"#,
            self.schema,
            use_script.trim_end(),
            field_indies.trim_end(),
            data_type.trim_end(),
            field_reader.trim_end(),
            field_writer.trim_end(),
            entity.trim_end(),
        )
    }

    fn build_use(&self) -> String {
        "use serbuffer::{types, BufferReader, BufferWriter, Buffer};\n".to_string()
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

    fn build_field_reader(&self) -> String {
        let mut field_read_method = "".to_string();
        for index in 0..self.fields.len() {
            let field = self.fields.get(index).unwrap();
            let method_script = match field.data_type {
                DataType::BYTES => format!(
                    r#"
    pub fn get_{}(&mut self) -> Result<&[u8], std::io::Error> {{
        self.reader.get_bytes({})
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
                DataType::BYTES => format!(
                    r#"
    pub fn set_{}(&mut self, {}: &[u8]) -> Result<(), std::io::Error> {{
        if self.writer_pos == {} {{
            self.writer_pos += 1;
            self.writer.set_bytes({})
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
                DataType::BYTES => {
                    ref_type = true;
                    fields = format!("{}\n    pub {}: &'a [u8],", fields, field.name);
                    writers = format!(
                        "{}\n        writer.set_bytes(self.{})?;",
                        writers, field.name
                    );
                    readers = format!(
                        "{}\n            {}: reader.get_bytes({})?,",
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

        format!(
            r#"
#[derive(Clone, Debug)]
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
            ref_type,
            fields.trim_start(),
            ref_type,
            ref_type,
            writers.trim_start(),
            readers.trim_start()
        )
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
    BYTES,
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
            DataType::BYTES => write!(f, "{}", "BYTES".to_lowercase()),
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
            "BYTES" => Ok(DataType::BYTES),
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
    use crate::{Codegen, DataType};

    #[test]
    pub fn code_gen_basic_type_test() {
        let script = Codegen::new("", "DemoSchema")
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
        let script = Codegen::new("", "DemoSchema")
            .field("timestamp", DataType::U64)
            .field("application_name", DataType::STRING)
            .field("agent_id", DataType::STRING)
            .field("a", DataType::BOOL)
            .field("a1", DataType::U8)
            .field("a2", DataType::BYTES)
            .build_script();

        println!("-- Start ---");
        println!("{}", script);
        println!("-- End ---");
    }
}
