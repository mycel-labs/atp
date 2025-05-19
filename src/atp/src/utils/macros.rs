#[macro_export]
macro_rules! generate_getters {
    ($($field:ident: $type:ty),*) => {
        $(
            pub fn $field(&self) -> &$type {
                &self.$field
            }
        )*
    };
}
