/// Macro to easily define a model that implements all necessary traits
///
/// This macro automatically implements the `Model` trait and `Storable` trait for your struct,
/// making it ready to use with ic-nosql database operations. It requires explicit specification
/// of the primary key for deterministic behavior.
///
/// # Examples
///
/// Model with only primary key (no secondary key):
/// ```
/// use ic_nosql::{define_model, CandidType};
/// use serde::{Deserialize, Serialize};
/// use candid::{Decode, Encode};
///
/// define_model! {
///     #[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
///     pub struct Post {
///         pub id: String,
///         pub title: String,
///         pub content: String,
///     }
///     
///     primary_key: id -> String,
/// }
/// ```
///
/// Model with both primary and secondary keys:
/// ```
/// use ic_nosql::{define_model, CandidType};
/// use serde::{Deserialize, Serialize};
/// use candid::{Decode, Encode};
///
/// define_model! {
///     #[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
///     pub struct User {
///         pub id: String,
///         pub username: String,
///         pub email: String,
///         pub created_at: u64,
///     }
///     
///     primary_key: id -> String,
///     secondary_key: username -> String,
/// }
/// ```
///
/// # Key Requirements
///
/// - **Primary key is required**: You must explicitly specify which field serves as the primary key
/// - **Secondary key is optional**: You can optionally specify a secondary key for indexed queries
/// - **Field types**: The specified fields must implement `Clone` and match the declared types
/// - **Deterministic behavior**: No ambiguity about which field is the primary key
#[macro_export]
macro_rules! define_model {
    // Model with only primary key (no secondary key)
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_attr:meta])*
                $field_vis:vis $field_name:ident: $field_type:ty
            ),* $(,)?
        }

        primary_key: $primary_field:ident -> $primary_type:ty,
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
            type SecondaryKey = ();

            fn get_primary_key(&self) -> Self::PrimaryKey {
                self.$primary_field.clone()
            }

            fn get_secondary_key(&self) -> Option<Self::SecondaryKey> {
                None
            }

            fn model_name() -> &'static str {
                stringify!($name)
            }
        }

        impl ic_stable_structures::Storable for $name {
            fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
               std::borrow::Cow::Owned(candid::Encode!(self).expect("Failed to encode model for storage"))
            }

            fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
               candid::Decode!(bytes.as_ref(), Self).expect("Failed to decode model from storage")
            }

            const BOUND: ic_stable_structures::storable::Bound =
                ic_stable_structures::storable::Bound::Unbounded;
        }
    };

    // Model with custom primary key and secondary key
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_attr:meta])*
                $field_vis:vis $field_name:ident: $field_type:ty
            ),* $(,)?
        }

        primary_key: $primary_field:ident -> $primary_type:ty,
        secondary_key: $secondary_field:ident -> $secondary_type:ty,
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
            type SecondaryKey = $secondary_type;

            fn get_primary_key(&self) -> Self::PrimaryKey {
                self.$primary_field.clone()
            }

            fn get_secondary_key(&self) -> Option<Self::SecondaryKey> {
                Some(self.$secondary_field.clone())
            }

            fn model_name() -> &'static str {
                stringify!($name)
            }
        }

        impl ic_stable_structures::Storable for $name {
            fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
                std::borrow::Cow::Owned(candid::Encode!(self).expect("Failed to encode model for storage"))
            }

            fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
                candid::Decode!(bytes.as_ref(), Self).expect("Failed to decode model from storage")
            }

            const BOUND: ic_stable_structures::storable::Bound =
                ic_stable_structures::storable::Bound::Unbounded;
        }
    };
}

pub use define_model;
