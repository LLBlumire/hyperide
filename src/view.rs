#[doc(hidden)]
pub use html_escape::encode_text;

pub trait IntoView {
    fn into_view(self) -> String;
}

macro_rules! impl_to_str {
    ($t:ty) => {
        impl IntoView for $t {
            fn into_view(self) -> String {
                self.to_string()
            }
        }
    };
    ($t:ty, $($r:ty),*) => {
        impl_to_str!($t);
        impl_to_str!($($r),*);
    };
}

macro_rules! impl_escaped_str {
    ($t:ty) => {
        impl IntoView for $t {
            fn into_view(self) -> String {
                $crate::view::html_escape(self.to_string())
            }
        }
    };
    ($t:ty, $($r:ty),*) => {
        impl_to_str!($t);
        impl_to_str!($($r),*);
    };
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
impl_escaped_str!(String, &str, std::borrow::Cow<'_, str>);

impl<T> IntoView for Option<T>
where
    T: IntoView,
{
    fn into_view(self) -> String {
        self.map(IntoView::into_view).unwrap_or_default()
    }
}
