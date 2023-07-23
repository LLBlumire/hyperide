use std::{borrow::Cow, ops::Deref};

#[doc(hidden)]
pub use html_escape::encode_text;

pub struct HyperText<'a> {
    inner: Cow<'a, str>,
}
impl<'a> Deref for HyperText<'a> {
    type Target = <Cow<'a, str> as Deref>::Target;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}
impl<'a> From<&'a str> for HyperText<'a> {
    fn from(value: &'a str) -> Self {
        HyperText {
            inner: Cow::Borrowed(value),
        }
    }
}
impl<'a> From<String> for HyperText<'a> {
    fn from(value: String) -> Self {
        HyperText {
            inner: Cow::Owned(value),
        }
    }
}
impl<'a> Default for HyperText<'a> {
    fn default() -> Self {
        Self {
            inner: Cow::Borrowed(""),
        }
    }
}

pub trait IntoHyperText<'a> {
    fn into_hyper_text(self) -> HyperText<'a>;
}

impl<'a> IntoHyperText<'a> for HyperText<'a> {
    fn into_hyper_text(self) -> HyperText<'a> {
        self
    }
}

macro_rules! impl_to_str {
    ($t:ty) => {
        impl<'a> IntoHyperText<'a> for $t {
            fn into_hyper_text(self) -> HyperText<'a> {
                self.to_string().into()
            }
        }
    };
    ($t:ty, $($r:ty),*) => {
        impl_to_str!($t);
        impl_to_str!($($r),*);
    };
}

impl<'a> IntoHyperText<'a> for &'a str {
    fn into_hyper_text(self) -> HyperText<'a> {
        self.into()
    }
}
impl<'a> IntoHyperText<'a> for String {
    fn into_hyper_text(self) -> HyperText<'a> {
        self.into()
    }
}
impl<'a> IntoHyperText<'a> for Cow<'a, str> {
    fn into_hyper_text(self) -> HyperText<'a> {
        HyperText { inner: self }
    }
}

impl_to_str![
    usize,
    u8,
    u16,
    u32,
    u64,
    u128,
    isize,
    i8,
    i16,
    i32,
    i64,
    i128,
    f32,
    f64,
    char,
    bool,
    std::net::IpAddr,
    std::net::SocketAddr,
    std::net::SocketAddrV4,
    std::net::SocketAddrV6,
    std::net::Ipv4Addr,
    std::net::Ipv6Addr,
    std::char::ToUppercase,
    std::char::ToLowercase,
    std::num::NonZeroI8,
    std::num::NonZeroU8,
    std::num::NonZeroI16,
    std::num::NonZeroU16,
    std::num::NonZeroI32,
    std::num::NonZeroU32,
    std::num::NonZeroI64,
    std::num::NonZeroU64,
    std::num::NonZeroI128,
    std::num::NonZeroU128,
    std::num::NonZeroIsize,
    std::num::NonZeroUsize,
    std::panic::Location<'_>,
    std::fmt::Arguments<'_>
];

impl<'a, T> IntoHyperText<'a> for Option<T>
where
    T: IntoHyperText<'a>,
{
    fn into_hyper_text(self) -> HyperText<'a> {
        self.map(IntoHyperText::into_hyper_text).unwrap_or_default()
    }
}
