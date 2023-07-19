#[doc(hidden)]
pub const SCRIPT: &str = include_str!("_hyperscript.min.js");

#[macro_export]
macro_rules! include_hyperscript {
    () => {
        $crate::hyperide! {
            <script _hr_no_raw=true>
                { $crate::hyperscript::SCRIPT }
            </script>
        }
    };
}

pub use include_hyperscript;
