/// Macro to easily define a model that implements all necessary traits
///
/// # Examples
///
/// Basic model definition:
/// ```
/// // Note: define_model! macro is currently disabled
/// // Manual implementation:
/// use ic_nosql::{CandidType, Deserialize, Serialize};
///
/// #[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
/// pub struct User {
///     pub id: String,
///     pub username: String,
///     pub email: String,
///     pub created_at: u64,
/// }
/// ```
///
/// Model with secondary key:
/// ```
/// // Note: define_model! macro is currently disabled
/// // Manual implementation would be needed
/// # use ic_nosql::{CandidType, Deserialize, Serialize};
/// # #[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
/// # pub enum AccountStatus { Active, Inactive }
/// # pub struct Account { pub id: String }
/// ```
#[macro_export]
macro_rules! define_model {
    // Basic model without secondary key
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_attr:meta])*
                $field_vis:vis $field_name:ident: $field_type:ty
            ),* $(,)?
        }
    ) => {
        $(#[$attr])*
        $vis struct $name {
            $(
                $(#[$field_attr])*
                $field_vis $field_name: $field_type,
            )*
        }

        impl $crate::traits::Model for $name {
            type PrimaryKey = String;
            type SecondaryKey = ();

            fn get_primary_key(&self) -> Self::PrimaryKey {
                // Assume the first field is the primary key
                define_model!(@get_first_field self $(,$field_name)*)
            }

            fn model_name() -> &'static str {
                stringify!($name)
            }
        }

        impl ic_stable_structures::Storable for $name {
            fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
                std::borrow::Cow::Owned(candid::Encode!(self).unwrap())
            }

            fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
                candid::Decode!(bytes.as_ref(), Self).unwrap()
            }

            const BOUND: ic_stable_structures::storable::Bound =
                ic_stable_structures::storable::Bound::Unbounded;
        }
    };

    // Model with custom primary key and optional secondary key
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_attr:meta])*
                $field_vis:vis $field_name:ident: $field_type:ty
            ),* $(,)?
        }

        primary_key: $primary_field:ident -> $primary_type:ty,
        $(secondary_key: $secondary_field:ident -> $secondary_type:ty,)?
    ) => {
        $(#[$attr])*
        $vis struct $name {
            $(
                $(#[$field_attr])*
                $field_vis $field_name: $field_type,
            )*
        }

        impl $crate::traits::Model for $name {
            type PrimaryKey = $primary_type;
            $(type SecondaryKey = $secondary_type;)?

            fn get_primary_key(&self) -> Self::PrimaryKey {
                self.$primary_field.clone()
            }

            $(
                fn get_secondary_key(&self) -> Option<Self::SecondaryKey> {
                    Some(self.$secondary_field.clone())
                }
            )?

            fn model_name() -> &'static str {
                stringify!($name)
            }
        }

        impl ic_stable_structures::Storable for $name {
            fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
                std::borrow::Cow::Owned(candid::Encode!(self).unwrap())
            }

            fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
                candid::Decode!(bytes.as_ref(), Self).unwrap()
            }

            const BOUND: ic_stable_structures::storable::Bound =
                ic_stable_structures::storable::Bound::Unbounded;
        }
    };

    // Helper to get the first field (used as default primary key)
    (@get_first_field $self:ident, $first:ident $(, $rest:ident)*) => {
        $self.$first.to_string()
    };
}

pub use define_model;
