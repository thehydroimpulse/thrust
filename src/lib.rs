#![allow(unused_imports, unused_variables, dead_code, unused_must_use, unused_mut)]
#![feature(associated_type_defaults, mpsc_select, question_mark)]

#[macro_use]
extern crate nom;
#[macro_use]
extern crate lazy_static;
extern crate mio;
extern crate byteorder;
extern crate tangle;
extern crate rand;
extern crate slab;
extern crate bytes;
extern crate num_cpus;
extern crate libc;

use nom::{IResult};
use std::str;
use std::convert;
use std::string;

mod util;
mod event_loop;
mod reactor;
pub mod protocol;
pub mod binary_protocol;
// mod service;
mod pipeline;
mod runner;
mod dispatcher;
mod result;
mod transport;

pub use result::{ThrustResult, ThrustError};
pub use protocol::{Serializer, Serialize, Deserialize, ThriftSerializer, ThriftDeserializer};
