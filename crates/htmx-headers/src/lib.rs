#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct AsciiHeaderValue(::http::HeaderValue);
impl AsciiHeaderValue {
    fn from_value(value: ::http::HeaderValue) -> Option<Self> {
        // SAFETY check for [`AsciiHeaderValue::as_str`]
        let bytes = value.as_bytes();
        for &b in bytes {
            if !(b >= 32 && b < 127 || b == b'\t') {
                return None;
            }
        }
        Some(AsciiHeaderValue(value))
    }

    fn from_str(s: &str) -> Option<Self> {
        ::http::HeaderValue::from_str(s).ok().map(AsciiHeaderValue)
    }

    fn as_str(&self) -> &str {
        let bytes = self.0.as_bytes();
        // SAFETY checked in [`AsciiHeaderValue::new`]
        unsafe { ::std::str::from_utf8_unchecked(bytes) }
    }

    fn into_value(self) -> ::http::HeaderValue {
        self.0
    }
}

macro_rules! str_header {
    ($name:ident, $n:ident = $s:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(crate::AsciiHeaderValue);
        pub static $n: ::headers::HeaderName = ::headers::HeaderName::from_static($s);
        impl $name {
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
            pub fn into_value(self) -> ::http::HeaderValue {
                self.0.into_value()
            }
            pub fn from_str(s: &str) -> Option<Self> {
                crate::AsciiHeaderValue::from_str(s).map($name)
            }
        }
        impl ::headers::Header for $name {
            fn name() -> &'static http::HeaderName {
                &$n
            }
            fn decode<'i, I>(values: &mut I) -> Result<Self, ::headers::Error>
            where
                Self: Sized,
                I: Iterator<Item = &'i ::http::HeaderValue>,
            {
                values
                    .next()
                    .and_then(|one| values.next().is_none().then_some(one))
                    .cloned() // Type is Arc-like so cheap to clone
                    .and_then(crate::AsciiHeaderValue::from_value)
                    .map($name)
                    .ok_or_else(::headers::Error::invalid)
            }

            fn encode<E: Extend<::http::HeaderValue>>(&self, values: &mut E) {
                values.extend(std::iter::once(self.clone().into_value()))
            }
        }
    };
}

macro_rules! true_header {
    ($name:ident, $n:ident = $s:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name;
        pub static $n: ::headers::HeaderName = ::headers::HeaderName::from_static($s);
        impl $name {
            pub fn into_value(self) -> ::http::HeaderValue {
                ::http::HeaderValue::from_static("true")
            }
        }
        impl ::headers::Header for $name {
            fn name() -> &'static http::HeaderName {
                &$n
            }
            fn decode<'i, I>(values: &mut I) -> Result<Self, ::headers::Error>
            where
                Self: Sized,
                I: Iterator<Item = &'i ::http::HeaderValue>,
            {
                values
                    .next()
                    .and_then(|one| values.next().is_none().then_some(one))
                    .filter(|value| value.as_bytes() == b"true")
                    .is_some()
                    .then_some($name)
                    .ok_or_else(::headers::Error::invalid)
            }

            fn encode<E: Extend<::http::HeaderValue>>(&self, values: &mut E) {
                values.extend(std::iter::once(self.clone().into_value()))
            }
        }
    };
}

pub mod request {
    true_header!(HxBoosted, HX_BOOSTED = "hx-boosted");
    str_header!(HxCurrentUrl, HX_CURRENT_URL = "hx-current-url");
    true_header!(
        HxHistoryRestoreRequest,
        HX_HISTORY_RESTORE_REQUEST = "hx-history-restore-request"
    );
    str_header!(HxPrompt, HX_PROMPT = "hx-prompt");
    true_header!(HxRequest, HX_REQUEST = "hx-request");
    str_header!(HxTarget, HX_TARGET = "hx-target");
    str_header!(HxTriggerName, HX_TRIGGER_NAME = "hx-trigger-name");
    str_header!(HxTrigger, HX_TRIGGER = "hx-trigger");
}

pub mod response {
    str_header!(HxLocation, HX_LOCATION = "hx-location");
    str_header!(HxPushUrl, HX_PUSH_URL = "hx-push-url");
    str_header!(HxRedirect, HX_REDIRECT = "hx-redirect");
    true_header!(HxRefresh, HX_REFRESH = "hx-refresh");
    str_header!(HxReplaceUrl, HX_REPLACE_URL = "hx-replace-url");
    str_header!(HxReswap, HX_RESWAP = "hx-reswap");
    str_header!(HxRetarget, HX_RETARGET = "hx-retarget");
    str_header!(HxReselect, HX_RESELECT = "hx-reselect");
    str_header!(HxTrigger, HX_TRIGGER = "hx-trigger");
    str_header!(
        HxTriggerAfterSettle,
        HX_TRIGGER_AFTER_SETTLE = "hx-trigger-after-settle"
    );
    str_header!(
        HxTriggerAfterSwap,
        HX_TRIGGER_AFTER_SWAP = "hx-trigger-after-swap"
    );
}
