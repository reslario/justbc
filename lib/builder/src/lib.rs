//! Provides the [builder_method!](crate::builder_method) and
//! [builder_methods!](crate::builder_methods) macros.
//!
//! See their respective documentation for more info.

/// Allows defining builder methods using a semicolon-separated list.
///
/// # Examples
/// 
///```
/// use builder::builder_methods;
///
/// #[derive(Default)]
/// struct S {
///     a: String
/// }
/// 
/// impl S {
///     builder_methods! {
///         a: String
///     }
/// }
/// 
/// assert_eq!(
///     "yes",
///     S::default().a("yes".into()).a
/// )
/// ```
///
/// The macro allows for a fair bit of customisation, including transforming arguments.
///
/// ```
/// use builder::builder_methods;
/// use std::borrow::Cow;
///
/// #[derive(Default)]
/// struct S<'a> {
///     a: u8,
///     b: Cow<'a, str>
/// }
/// 
/// impl <'a> S<'a> {
///     builder_methods! {
///         /// Sets a.
///         #[inline(always)]
///         a: u8;
///         #[inline]
///         pub b: impl Into<Cow<'a, str>> => b.into()
///     }
/// }
/// 
/// assert!(matches!(
///     S::default().a(2).b("yes"),
///     S { a: 2, b: Cow::Borrowed("yes") }
/// ))
/// ```
#[macro_export]
macro_rules! builder_methods {
    ($($(#[$attr:meta])* $v:vis $field:ident: $t:ty $( => $f:expr)?);+) => {
        $($crate::builder_method! { $(#[$attr])* $v $field: $t $( => $f)? })+
    }
}

/// The same as [builder_methods!](crate::builder_methods), but only allows
/// defining one method at a time.
#[macro_export]
macro_rules! builder_method {
    ($(#[$attr:meta])* $v:vis $field:ident: $t:ty) => {
        $(#[$attr])*
        $v fn $field(self, $field: $t) -> Self {
            Self { $field, ..self }
        }
    };

    ($(#[$attr:meta])* $v:vis $field:ident: $t:ty => $f:expr) => {
        $(#[$attr])*
        $v fn $field(self, $field: $t) -> Self {
            Self { $field: $f, ..self }
        }
    }
}
