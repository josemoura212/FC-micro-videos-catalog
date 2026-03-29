#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::correctness,
    clippy::perf,
    clippy::style,
    clippy::suspicious,
    clippy::complexity,
    clippy::nursery,
    clippy::unwrap_used,
    unused_qualifications,
    rust_2024_compatibility,
    trivial_casts,
    trivial_numeric_casts,
    unused_allocation,
    clippy::unnecessary_cast,
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::dbg_macro,
    clippy::deprecated_cfg_attr,
    clippy::separated_literal_suffix,
    deprecated
)]
#![forbid(unsafe_code, deprecated_in_future)]

pub mod domain;
pub mod infrastructure;
