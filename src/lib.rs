extern crate self as hyperide;

pub use hyperide_macro::hyperide;

pub mod htmx;
pub mod hyperscript;
pub mod tailwind;
pub mod vercel;

mod attr;
pub use attr::IntoAttrText;

mod hyper;
pub use hyper::HyperText;
pub use hyper::IntoHyperText;

/// Bakes css from a file into hyperide. Will insert it inside `<style>`
/// tags and allows you to write styles in a `.css` file but include it in
/// generated HTML without needing to serve the file separately and causing an
/// additional request from the client.
///
/// ```rust
/// # use hyperide_macro::hyperide;
/// hyperide! {
///     include_style!("foo.css")
/// };
/// ```
#[macro_export]
macro_rules! include_style {
    ($file:expr $(,)?) => {{
        $crate::hyperide! {
            <style _hr_no_raw=true>
                { std::include_str!($file) }
            </style>
        }
    }};
}

/// Bakes javascript from a file into hyperide. Will insert it inside `<script>`
/// tags and allows you to write javascript in a `.js` file but include it in
/// generated HTML without needing to serve the file separately and causing an
/// additional request from the client.
///
/// ```no_run
/// # use hyperide::hyperide;
/// let my_script = hyperide! {
///     include_script!("foo.js")
/// };
/// ```
#[macro_export]
macro_rules! include_script {
    ($file:expr $(,)?) => {
        $crate::hyperide! {
            <script _hr_no_raw=true>
                { include_str!($file) }
            </script>
        }
    };
}
