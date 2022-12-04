#![feature(iterator_try_collect)]
#![warn(
    anonymous_parameters,
    missing_copy_implementations,
    missing_debug_implementations,
    rust_2018_idioms,
    rustdoc::private_doc_tests,
    trivial_casts,
    trivial_numeric_casts,
    unused,
    future_incompatible,
    nonstandard_style,
    unsafe_code,
    unused_import_braces,
    unused_results,
    variant_size_differences
)]
#![allow(opaque_hidden_inferred_bound)]
/// opaque_hidden_inferred_bound is needed because there is an implied bound of
/// `warp::generic::Tuple`, which is private.
pub mod route;
pub mod service;
