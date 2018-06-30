#![feature(const_int_ops)]
#![feature(vec_resize_with)]

#[macro_use]
extern crate log;
extern crate arrayvec;
extern crate num_traits;

extern crate shine_store as store;

pub mod bitset;
pub mod smat;
pub mod svec;
