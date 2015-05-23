#![feature(slice_patterns, plugin_registrar, plugin, custom_derive, rustc_private)]
#![allow(unused_variable, dead_code)]
#![plugin(serde_macros)]

// extern crate syntax;
// extern crate rustc;

// use syntax::codemap::Span;
// use syntax::parse::token;
// use syntax::ast::{TokenTree, TtToken};
// use syntax::ext::base::{ExtCtxt, MacResult, DummyResult, MacEager};
// use syntax::ext::build::AstBuilder;  // trait for expr_usize
// use rustc::plugin::Registry;

#[macro_use]
extern crate nom;
extern crate serde;
extern crate byteorder;

// mod parser;
mod server;
mod transport;
mod network;
mod protocol;

pub use server::{Service, Server};

// fn expand_thrust(cx: &mut ExtCtxt, sp: Span, args: &[TokenTree])
//     -> Box<MacResult + 'static> {

//     quote_expr(
// }

// #[plugin_registrar]
// pub fn plugin_registrar(reg: &mut Registry) {
//     reg.register_macro("thrust", expand_thrust);
// }
