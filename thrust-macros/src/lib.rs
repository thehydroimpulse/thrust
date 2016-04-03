#![crate_type="dylib"]
#![feature(plugin_registrar, quote, rustc_private, question_mark, concat_idents)]

extern crate thrust_parser;
extern crate syntax;
extern crate rustc;
extern crate rustc_plugin;

use thrust_parser::{Ast, Parser};
use syntax::util::small_vector::SmallVector;
use std::mem;
use std::iter::Iterator;
use syntax::codemap::Span;
use syntax::fold::Folder;
use syntax::parse::{self, parser};
use syntax::parse::token::{self, Token};
use syntax::parse::token::keywords;
use syntax::print::pprust;
use syntax::ast::{self, TokenTree};
use syntax::ptr::P;
use syntax::ext::base::{ExtCtxt, MacResult, DummyResult, MacEager};
use syntax::ext::build::AstBuilder;  // trait for expr_usize
use rustc_plugin::Registry;

pub enum State {
    Begin
}

pub struct Compiler<'a: 'x, 'x> {
    inner: parser::Parser<'a>,
    cx: &'x mut ExtCtxt<'a>,
    state: State
}

impl<'a, 'x> Compiler<'a, 'x> {
    pub fn new(cx: &'x mut ExtCtxt<'a>, args: &[TokenTree]) -> Compiler<'a, 'x> {
        Compiler::<'a, 'x> {
            inner: cx.new_parser_from_tts::<'a>(args),
            cx: cx,
            state: State::Begin
        }
    }

    pub fn parse(&mut self) -> Option<String> {
        if let Ok(expr) = self.inner.parse_expr() {
            let entry = self.cx.expander().fold_expr(expr);
            let th = match entry.node {
                ast::ExprKind::Lit(ref lit) => {
                    match lit.node {
                        ast::LitKind::Str(ref s, _) => s.to_string(),
                        _ => {
                            self.cx.span_err(entry.span, &format!(
                             "expected string literal but got `{}`",
                             pprust::lit_to_string(&**lit)));
                            return None;
                        }
                    }
                },
                _ => {
                    self.cx.span_err(entry.span, &format!(
                    "expected string literal but got `{}`",
                    pprust::expr_to_string(&*entry)));
                    return None
                }
            };
            if !self.inner.eat(&token::Eof) {
                self.cx.span_err(self.inner.span, "only one string literal allowed");
                return None;
            }

            Some(th)
        } else {
            self.cx.parse_sess().span_diagnostic.err("failure parsing token tree");
            return None;
        }
    }

    pub fn code(&mut self, input: String) -> Result<P<ast::Item>, thrust_parser::Error> {
        let mut parser = Parser::new(&input);
        let mut items = Vec::new();

        // We expect a namespace to appear first.
        let ns = parser.parse_namespace()?;
        let module = token::str_to_ident(&ns.module);

        while let Ok(node) = parser.parse_item() {
            match node.ir(self.cx) {
                Some(item) => items.push(item),
                // The node didn't want to export an item. All good!
                None => {}
            }
        }

        // pieces.push(parser.parse_enum()?.ir(self.cx));
        // pieces.push(parser.parse_struct()?.ir(self.cx));

        Ok(quote_item!(self.cx, pub mod $module {
            $items
        }).unwrap())
    }

    fn get_ident_from_pat(&mut self, pat: P<ast::Pat>) -> ast::Ident {
        match pat.node {
            ast::PatKind::Ident(mode, ref span, ref p) => {
                span.node
            },
            _ => panic!("Error")
        }
    }
}

fn expand_rn(cx: &mut ExtCtxt, sp: Span, args: &[TokenTree]) -> Box<MacResult + 'static> {
    if args.len() == 0 {
        // XXX: Return an empty future.
        return DummyResult::any(sp);
    }

    let mut compiler = Compiler::new(cx, args);
    let input = match compiler.parse() {
        Some(s) => s,
        None => panic!("Expected a string")
    };

    MacEager::items(SmallVector::one(compiler.code(input).unwrap()))
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("thrust", expand_rn);
}
