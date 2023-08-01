#[doc(hidden)]
pub const SCRIPT: &str = include_str!("htmx.min.js");

#[macro_export]
macro_rules! include_htmx {
    () => {
        $crate::hyperide! {
            <script _hr_no_raw=true>
                { $crate::htmx::SCRIPT }
            </script>
        }
    };
}

pub use include_htmx;

pub mod headers {
    pub use htmx_headers::*;
}
