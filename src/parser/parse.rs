use super::Scope;

use super::ast::*;
use super::Parser;
use crate::lexer::Token;
use crate::source::{ErrKind, Ident};

pub trait ParserError {}

pub trait Parse {
    fn parse_prog(&mut self) -> Vec<Expr>;
    fn parse_level(&mut self, level: u8) -> Expr;

    fn parse_member(&mut self) -> Expr;
    fn parse_call_fn(&mut self) -> Expr;

    fn parse_expr(&mut self) -> Expr;
    fn parse_declare(&mut self) -> Expr;
    fn parse_declare_fn(&mut self, id: Ident) -> Expr;
    fn parse_if_expr(&mut self) -> Expr;
    fn parse_while_expr(&mut self) -> Expr;
    fn parse_ret_expr(&mut self) -> Expr;

    fn parse_body(&mut self) -> Vec<Expr>;
    fn parse_list(&mut self) -> Vec<Expr>;
}

impl Parse for Parser {
    fn parse_prog(&mut self) -> Vec<Expr> {
        let mut body: Vec<Expr> = Vec::new();
        while self.current() != Token::EOF {
            self.current_scope = Scope::Top;
            let mut expr = self.parse_level(0);

            if !self.current_scope.is_used() {
                expr = Expr::Discard(Box::new(expr));
            }

            body.push(expr);
        }

        return body;
    }

    fn parse_level(&mut self, level: u8) -> Expr {
        let mut left: Expr = self.parse_member();
        let mut right: Expr;

        loop {
            // 5 (2*) 5 nothing (1+) 5
            if let Token::Operator(c) = self.current() {
                if c == "=" {
                    self.next();
                    self.current_scope = Scope::Value;
                    let right = self.parse_level(0);

                    left = Expr::VarAssign {
                        name: Box::new(left),
                        val: Box::new(right),
                    };
                    break;
                }

                let current_op_level = get_operator_level(c.as_str());
                if current_op_level < level {
                    break;
                }

                self.next();
                right = self.parse_level(current_op_level + 1);

                left = Expr::BinaryExpr {
                    op: c,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        return left;
    }
    fn parse_member(&mut self) -> Expr {
        let left = self.parse_call_fn();

        if self.current() == Token::Dot {
            self.next();
            let right = self.parse_call_fn();
            Expr::MemberExpr {
                parent: Box::new(left),
                child: Box::new(right),
            }
        } else {
            left
        }
    }

    fn parse_call_fn(&mut self) -> Expr {
        let call = self.parse_expr();

        if self.current() == Token::Colon {
            self.next();
            let args = self.parse_list();
            return Expr::FnCall {
                name: Box::new(call),
                args,
            };
        } else if self.current() == Token::Exec {
            self.next();
            return Expr::FnCall {
                name: Box::new(call),
                args: Vec::new(),
            };
        }

        return call;
    }

    fn parse_list(&mut self) -> Vec<Expr> {
        let mut args: Vec<Expr> = Vec::new();

        args.push(self.parse_level(0));
        while self.current() == Token::Comma {
            self.next();
            args.push(self.parse_level(0));
        }

        return args;
    }

    fn parse_expr(&mut self) -> Expr {
        let tok = self.current();
        match tok {
            Token::Int(i) => {
                self.next();
                Expr::Literal(Literal::Int(i))
            }
            Token::Float(f) => {
                self.next();
                Expr::Literal(Literal::Float(f))
            }
            Token::Bool(val) => {
                self.next();
                Expr::Literal(Literal::Bool(val))
            }
            Token::Str(s) => {
                self.next();
                Expr::Literal(Literal::Str(s))
            }

            Token::Err(_) => {
                todo!()
            }

            Token::Ident(id) => {
                self.next();
                Expr::Ident(Ident { val: id, tag: None })
            }
            // Token::Tag(tag) => {
            //     self.next();
            //     if let Token::Ident(id) = self.current() {
            //         self.next();
            //         return Expr::Ident(Ident {
            //             tag: Some(tag.to_string()),
            //             val: id,
            //         });
            //     }
            //     todo!()
            // }
            Token::LeftParen => {
                self.next();
                let expr = self.parse_level(0);
                self.except(Token::RightParen);
                expr
            }

            Token::LeftBrace => {
                self.next();
                let values = self.parse_list();
                self.except(Token::RightBrace);
                Expr::ListExpr(values)
            }

            Token::SetKw => self.parse_declare(),
            Token::WhileKw => self.parse_while_expr(),
            Token::IfKw => self.parse_if_expr(),
            Token::RetKw => self.parse_ret_expr(),
            _ => {
                self.err(
                    ErrKind::UnexceptedTokenE,
                    format!("unexcepted token [{:#?}]", tok),
                );
                self.next();

                // todo!(); // add null
                Expr::Literal(Literal::Int(0))
            }
        }
    }

    fn parse_declare(&mut self) -> Expr {
        self.next(); // n->t

        let left = self.parse_expr();
        self.current_scope = Scope::Value;
        if let Expr::Ident(name) = left {
            if Token::Operator("=".to_string()) == self.current() {
                self.next();

                let expr = self.parse_level(0);
                return Expr::VarDeclare {
                    name,
                    val: Box::new(expr),
                };
            }

            return self.parse_declare_fn(name);
        } else {
            self.err(
                ErrKind::UnexceptedTokenE,
                format!(
                    "unexcept token in set expression [{:?}] excepted an id",
                    left
                ),
            );

            left
        }
    }
    fn parse_declare_fn(&mut self, id: Ident) -> Expr {
        let mut id_args: Vec<Ident> = Vec::new();

        if self.current() == Token::Colon {
            self.next();
            let args = self.parse_list();

            for arg in args {
                if let Expr::Ident(id) = arg {
                    id_args.push(id);
                } else {
                    self.err(
                        ErrKind::UnexceptedArgs,
                        "excepted an id for arg".to_string(),
                    );
                    return self.parse_level(0);
                }
            }
        } else {
            self.except(Token::Exec);
        }
        let body = self.parse_body();

        self.push_function(id.clone(), id_args, body);
        self.current_scope = Scope::Value;
        Expr::PosInfo(id.val, self.line, self.column)
    }

    fn parse_if_expr(&mut self) -> Expr {
        self.next(); // remove if
        self.current_scope = Scope::Value;
        let condition = self.parse_level(0);
        let body = self.parse_body();

        let mut alt: Option<Box<Expr>> = None;
        if self.current() == Token::ElseKw {
            self.next();
            if self.current() == Token::IfKw {
                alt = Some(Box::new(self.parse_if_expr()));
            } else {
                alt = Some(Box::new(Expr::Block(self.parse_body())));
            }
        }

        Expr::IfExpr {
            condition: Box::new(condition),
            body,
            alt,
        }
    }
    fn parse_while_expr(&mut self) -> Expr {
        self.next();
        self.current_scope = Scope::Value;
        let condition = self.parse_level(0);
        let body = self.parse_body();

        Expr::WhileExpr {
            condition: Box::new(condition),
            body,
        }
    }

    #[inline]
    fn parse_body(&mut self) -> Vec<Expr> {
        let mut body: Vec<Expr> = vec![];

        self.except(Token::LeftBracket);
        while self.current() != Token::RightBracket && self.current() != Token::EOF {
            self.current_scope = Scope::Top;
            let mut expr = self.parse_level(0);

            if !self.current_scope.is_used() {
                expr = Expr::Discard(Box::new(expr));
            }

            body.push(expr);
        }
        self.except(Token::RightBracket);

        body
    }

    fn parse_ret_expr(&mut self) -> Expr {
        self.next();
        self.current_scope = Scope::Value;
        let expr = self.parse_level(0);
        Expr::RetExpr(Box::new(expr))
    }
}
