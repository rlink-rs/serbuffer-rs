pub struct FieldMetadata<const N: usize> {
    field_type: &'static [u8; N],
    field_name: &'static [&'static str; N],
}

impl<const N: usize> FieldMetadata<N> {
    pub const fn new(field_type: &'static [u8; N], field_name: &'static [&'static str; N]) -> Self {
        FieldMetadata {
            field_type,
            field_name,
        }
    }

    pub fn field_type(&self) -> &'static [u8] {
        &self.field_type[..]
    }

    pub fn field_name(&self) -> &'static [&'static str] {
        &self.field_name[..]
    }
}
