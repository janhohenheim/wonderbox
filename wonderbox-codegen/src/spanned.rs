//! Shamelessly stolen from [mockiato <3](https://github.com/myelin-ai/mockiato/blob/master/crates/mockiato-codegen/src/spanned.rs).

use proc_macro::Span;
use syn::spanned::Spanned;

pub(crate) trait SpannedUnstable {
    /// Returns a [`proc_macro::Span`] which can be
    /// used to print diagnostic messages.
    fn span_unstable(&self) -> Span;
}

impl<T> SpannedUnstable for T
where
    T: Spanned,
{
    fn span_unstable(&self) -> Span {
        // Turns a `Span` from `syn` into a span from `proc_macro`.
        // Note that this API is only available on nightly
        self.span().unstable()
    }
}
