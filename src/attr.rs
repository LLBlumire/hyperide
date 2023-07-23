use crate::{hyper::HyperText, IntoHyperText};

pub trait IntoAttrText<'a> {
    fn into_attr_text(self, attr: impl IntoHyperText<'a>) -> HyperText<'a>;
}

macro_rules! impl_to_attr {
    ($t:ty) => {
        impl<'a> IntoAttrText<'a> for $t {
            fn into_attr_text(self, attr: impl IntoHyperText<'a>) -> HyperText<'a> {
                let attr: &str = &attr.into_hyper_text();
                format!("{attr}=\"{self}\"").into()
            }
        }
    };
    ($t:ty, $($r:ty),*) => {
        impl_to_attr!($t);
        impl_to_attr!($($r),*);
    };
}
impl_to_attr![
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
    std::fmt::Arguments<'_>,
    String,
    &str,
    std::borrow::Cow<'_, str>
];

impl<'a> IntoAttrText<'a> for bool {
    fn into_attr_text(self, attr: impl IntoHyperText<'a>) -> HyperText<'a> {
        if self {
            attr.into_hyper_text()
        } else {
            HyperText::from("")
        }
    }
}

impl<'a, T> IntoAttrText<'a> for Option<T>
where
    T: IntoAttrText<'a>,
{
    fn into_attr_text(self, attr: impl IntoHyperText<'a>) -> HyperText<'a> {
        match self {
            Some(value) => value.into_attr_text(attr),
            None => HyperText::from(""),
        }
    }
}
