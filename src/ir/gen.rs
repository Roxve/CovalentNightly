use super::{Codegen, IROp};

use crate::{
    analysis::{AnalyzedExpr, TypedExpr},
    source::{ConstType, ErrKind, Ident},
};

type IR = Vec<IROp>;
type IRRes = Result<IR, u8>;

pub trait IRGen {
    fn gen_prog(&mut self, exprs: Vec<TypedExpr>) -> IR;
    fn gen_func(
        &mut self,
        name: String,
        args: Vec<Ident>,
        ret: ConstType,
        body: Vec<TypedExpr>,
    ) -> IRRes;
    fn gen_expr(&mut self, expr: TypedExpr) -> IRRes;

    fn gen_var_declare(&mut self, name: String, expr: TypedExpr) -> IRRes;
    fn gen_var_assign(&mut self, name: String, expr: TypedExpr) -> IRRes;
    fn gen_binary_expr(
        &mut self,
        ty: ConstType,
        op: String,
        left: TypedExpr,
        right: TypedExpr,
    ) -> IRRes;
}

impl IRGen for Codegen {
    fn gen_prog(&mut self, exprs: Vec<TypedExpr>) -> IR {
        let mut gen = vec![];

        for expr in exprs {
            let compiled_expr = self.gen_expr(expr);
            if compiled_expr.is_ok() {
                gen.append(&mut compiled_expr.unwrap());
            }
        }
        gen
    }
    fn gen_func(
        &mut self,
        name: String,
        args: Vec<Ident>,
        ret: ConstType,
        body: Vec<TypedExpr>,
    ) -> IRRes {
        for arg in &args {
            // types arent needed in ir gen
            self.env.add(&arg.val, ConstType::Dynamic);
        }
        let mut exprs = vec![];

        for expr in body {
            exprs.append(&mut self.gen_expr(expr)?);
        }

        Ok(vec![IROp::Def(ret, name, args, exprs)])
    }

    fn gen_expr(&mut self, expr: TypedExpr) -> IRRes {
        match expr.expr {
            AnalyzedExpr::Import { module, name, args } => {
                Ok(vec![IROp::Import(expr.ty, module, name, args)])
            }

            AnalyzedExpr::Func {
                ret,
                name,
                args,
                body,
            } => self.gen_func(name, args, ret, body),

            AnalyzedExpr::Literal(lit) => Ok(vec![IROp::Const(expr.ty, lit)]),

            AnalyzedExpr::BinaryExpr { op, left, right } => {
                self.gen_binary_expr(expr.ty, op, *left, *right)
            }
            AnalyzedExpr::VarDeclare { name, val } => self.gen_var_declare(name, *val),
            AnalyzedExpr::VarAssign { name, val } => self.gen_var_assign(name, *val),
            AnalyzedExpr::Id(name) => Ok(vec![IROp::Load(expr.ty, name)]),
            AnalyzedExpr::FnCall { name, args } => {
                let mut res: Vec<IROp> = vec![];
                let count = args.len().clone() as u16;

                for arg in args {
                    res.append(&mut self.gen_expr(arg)?);
                }
                res.push(IROp::Call(expr.ty, name, count));

                Ok(res)
            }
            AnalyzedExpr::RetExpr(expr) => {
                let mut res = vec![];
                let mut compiled_expr = self.gen_expr(*expr.clone())?;

                res.append(&mut compiled_expr);
                res.push(IROp::Ret(expr.ty));
                Ok(res)
            }

            AnalyzedExpr::As(conv) => {
                let mut res = vec![];
                let mut inside = self.gen_expr(*conv.clone())?;

                res.append(&mut inside);
                res.push(IROp::Conv(expr.ty, (*conv).ty));
                Ok(res)
            }

            AnalyzedExpr::Debug(_, _, _) => Ok(vec![]),
            AnalyzedExpr::Discard(dis) => {
                let mut compiled = self.gen_expr(*dis.clone())?;
                if dis.ty != ConstType::Void {
                    compiled.append(&mut vec![IROp::Pop]);
                }
                Ok(compiled)
            }

            AnalyzedExpr::If { cond, body, alt } => {
                let mut cond = self.gen_expr(*cond)?;

                let mut compiled_body = vec![];
                for expr in body {
                    compiled_body.append(&mut self.gen_expr(expr)?);
                }

                let alt = if alt.is_none() {
                    vec![]
                } else {
                    self.gen_expr(*alt.unwrap())?
                };

                let mut res = Vec::new();
                res.append(&mut cond);
                res.push(IROp::If(expr.ty, compiled_body, alt));
                Ok(res)
            }
            AnalyzedExpr::Block(block) => {
                let mut compiled_block = vec![];
                for expr in block {
                    compiled_block.append(&mut self.gen_expr(expr)?);
                }
                Ok(compiled_block)
            }
        }
    }

    fn gen_var_declare(&mut self, name: String, expr: TypedExpr) -> IRRes {
        let mut res = vec![];
        let mut g = self.gen_expr(expr.clone())?;
        let ty = expr.ty;

        res.push(IROp::Alloc(ty.clone(), name.clone()));
        self.env.add(&name, ty.clone());
        res.append(&mut g);
        res.push(IROp::Store(ty, name));

        Ok(res)
    }

    fn gen_var_assign(&mut self, name: String, expr: TypedExpr) -> IRRes {
        if !self.env.has(&name) {
            self.err(
                ErrKind::UndeclaredVar,
                format!("var {} is not declared", name.clone()),
            );
            return Err(ErrKind::UndeclaredVar as u8);
        }

        let mut res = vec![];
        let mut compiled_expr = self.gen_expr(expr.clone())?;
        let ty = expr.ty;

        if &self.env.get_ty(&name).unwrap() != &ty {
            res.push(IROp::Dealloc(self.env.get_ty(&name).unwrap(), name.clone()));
            res.push(IROp::Alloc(ty.clone(), name.clone()));

            self.env.modify(&name, ty.clone());
        }

        res.append(&mut compiled_expr);
        res.push(IROp::Store(ty, name));
        Ok(res)
    }

    fn gen_binary_expr(
        &mut self,
        ty: ConstType,
        op: String,
        left: TypedExpr,
        right: TypedExpr,
    ) -> IRRes {
        let mut res: IR = vec![];
        let mut lhs = self.gen_expr(left.clone())?;
        let mut rhs = self.gen_expr(right)?;
        res.append(&mut rhs);
        res.append(&mut lhs);
        if op.as_str() == "<" || op.as_str() == "<=" {
            res.reverse();
        }
        res.append(&mut vec![match op.as_str() {
            "+" => IROp::Add(ty),
            "-" => IROp::Sub(ty),
            "*" => IROp::Mul(ty),
            "/" => IROp::Div(ty),
            ">" | "<" => IROp::Comp(ConstType::Bool),
            ">=" | "<=" => IROp::EComp(ConstType::Bool),
            "==" => IROp::Eq(ConstType::Bool),
            o => todo!("add op {}", o),
        }]);
        Ok(res)
    }
}
