use crate::parser::ast::Expr;
use crate::parser::Function;
use crate::source::{ATErr, ConstType, ErrKind, Ident};

use super::*;

#[inline]
fn ty_as(ty: &ConstType, expr: TypedExpr) -> TypedExpr {
    TypedExpr {
        expr: AnalyzedExpr::As(Box::new(expr)),
        ty: ty.clone(),
    }
}

impl Analyzer {
    pub fn err(&mut self, kind: ErrKind, msg: String) {
        let err = ATErr {
            kind,
            msg,
            line: self.line,
            column: self.column,
        };
        err.out_error();
    }

    pub fn analyz_prog(
        exprs: Vec<Expr>,
        functions: Vec<Function>,
    ) -> Result<Vec<TypedExpr>, ErrKind> {
        let mut analyzer = Analyzer::new();
        let mut analyzed_prog = Vec::new();
        analyzer.env.push_function(
            Ident {
                tag: None,
                val: format!("writeln"),
            },
            vec![Ident {
                val: "x".to_string(),
                tag: Some(ConstType::Dynamic),
            }],
            ConstType::Void,
        );

        for func in functions {
            let args = func.args;

            for arg in &args {
                analyzer
                    .env
                    .add(&arg.val, arg.tag.clone().unwrap_or(ConstType::Dynamic), 0);
            }
            let mut body = vec![];
            for expr in func.body {
                body.push(analyzer.analyz(expr)?);
            }

            let ty = get_fn_type(&body);

            analyzer
                .env
                .push_function(func.name.clone(), args.clone(), ty);

            let expr = AnalyzedExpr::Func {
                ret: ty,
                name: func.name.val,
                args,
                body,
            };

            analyzed_prog.push(TypedExpr { expr, ty });
        }

        for expr in exprs {
            let analyzed_expr = analyzer.analyz(expr)?;
            analyzed_prog.push(analyzed_expr);
        }
        Ok(analyzed_prog)
    }

    pub fn analyz(&mut self, expr: Expr) -> Result<TypedExpr, ErrKind> {
        match expr {
            Expr::Literal(literal) => {
                let ty = match literal {
                    Literal::Int(_) => ConstType::Int,
                    Literal::Float(_) => ConstType::Float,
                    _ => todo!("Add literal {:?}", literal),
                };
                Ok(TypedExpr {
                    expr: AnalyzedExpr::Literal(literal),
                    ty,
                })
            }
            Expr::BinaryExpr { op, left, right } => self.analyz_binary_expr(*left, *right, op),
            Expr::Ident(id) => self.analyz_id(id),
            Expr::VarDeclare { name, val } => self.analyz_var_declare(name, *val),
            Expr::VarAssign { name, val } => self.analyz_var_assign(name, *val),
            Expr::Discard(expr) => {
                let ty = ConstType::Void;
                let expr = self.analyz(*expr)?;
                let expr = AnalyzedExpr::Discard(Box::new(expr));

                Ok(TypedExpr { expr, ty })
            }
            Expr::PosInfo(x, line, column) => {
                self.line = line;
                self.column = column;
                Ok(TypedExpr {
                    expr: AnalyzedExpr::Debug(x, line, column),
                    ty: ConstType::Void,
                })
            }

            Expr::RetExpr(expr) => {
                let expr = self.analyz(*expr)?;
                let ty = expr.ty;

                let expr = AnalyzedExpr::RetExpr(Box::new(expr));
                Ok(TypedExpr { expr, ty })
            }

            Expr::FnCall { name, args: params } => {
                let func = self.env.get_function(&name);
                if func.is_none() {
                    self.err(
                        ErrKind::UndeclaredVar,
                        format!("undeclared function {}", &name.val),
                    );
                    return Err(ErrKind::UndeclaredVar);
                }

                if &func.as_ref().unwrap().args.len() != &params.len() {
                    self.err(ErrKind::UndeclaredVar, format!("not enough arguments got {} arguments, expected {} arguments for function {}", params.len(), func.unwrap().args.len(), name.val));
                    return Err(ErrKind::UndeclaredVar);
                }

                let ty = self.env.get_ty(&name.val).unwrap();

                let mut args = vec![];
                for param in params {
                    args.push(TypedExpr {
                        expr: AnalyzedExpr::As(Box::new(self.analyz(param)?)),
                        ty: ConstType::Dynamic,
                    });
                }

                let expr = AnalyzedExpr::FnCall {
                    name: name.val,
                    args,
                };

                Ok(TypedExpr { expr, ty })
            }
            _ => todo!("add typed expr {:?}", expr),
        }
    }

    pub fn analyz_binary_expr(
        &mut self,
        left: Expr,
        right: Expr,
        op: String,
    ) -> Result<TypedExpr, ErrKind> {
        let mut lhs = self.analyz(left)?;
        let mut rhs = self.analyz(right)?;

        if lhs.ty != rhs.ty {
            if lhs.ty == ConstType::Float && rhs.ty == ConstType::Int {
                rhs = ty_as(&lhs.ty, rhs);
            } else if lhs.ty == ConstType::Int && rhs.ty == ConstType::Float {
                lhs = ty_as(&rhs.ty, lhs);
            } else if lhs.ty == ConstType::Dynamic {
                rhs = ty_as(&lhs.ty, rhs);
            } else if rhs.ty == ConstType::Dynamic {
                lhs = ty_as(&rhs.ty, lhs);
            }
        }
        let ty = lhs.ty.clone();
        let left = Box::new(lhs);
        let right = Box::new(rhs);

        let expr = AnalyzedExpr::BinaryExpr { op, left, right };
        Ok(TypedExpr { expr, ty })
    }

    pub fn analyz_id(&mut self, id: Ident) -> Result<TypedExpr, ErrKind> {
        if !self.env.has(&id.val) {
            return Err(ErrKind::UndeclaredVar);
        }

        let rc = self.env.get_rc(&id.val).unwrap() + 1;
        let ty = self.env.get_ty(&id.val).unwrap();
        self.env.modify_rc(&id.val, rc);

        let expr = AnalyzedExpr::Id(id.val, rc);
        Ok(TypedExpr { expr, ty })
    }

    pub fn analyz_var_declare(&mut self, name: Ident, val: Expr) -> Result<TypedExpr, ErrKind> {
        let val = self.analyz(val)?;
        let name = name.val;
        if self.env.has(&name) {
            return Err(ErrKind::VarAlreadyDeclared);
        }
        let ty = val.ty.clone();
        self.env.add(&name, ty.clone(), 0);

        let expr = AnalyzedExpr::VarDeclare {
            name,
            val: Box::new(val),
        };
        Ok(TypedExpr { expr, ty })
    }

    pub fn analyz_var_assign(&mut self, name: Ident, val: Expr) -> Result<TypedExpr, ErrKind> {
        let val = self.analyz(val)?;
        let name = name.val;
        if !self.env.has(&name) {
            return Err(ErrKind::UndeclaredVar);
        }
        let ty = val.ty.clone();
        let rc = self.env.get_rc(&name).unwrap() + 1;
        self.env.modify(&name, ty.clone());

        self.env.modify_rc(&name, rc);

        let expr = AnalyzedExpr::VarAssign {
            name,
            val: Box::new(val),
        };
        Ok(TypedExpr { expr, ty })
    }
}
