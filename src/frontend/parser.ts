import { Stmt, Program, Expr } from "./AST/stmts.ts";
import { ParserExpr } from "./parse/expr.ts";
import { Ion, Type } from "./Ion.ts"


export class Parser extends ParserExpr {
   public productAST() : Program {
     let program: Program = {
       type: "Program",
       body: [],
       line: this.line,
       colmun: this.colmun
     } as Program
     let lastevaluated: any;
     while(this.notEOF()) {
        lastevaluated = program.body.push(this.parse_stmt());
     }
     return program;  
   }

   protected parse_stmt() : Stmt {
     switch(this.at().type) {
       default:
         return this.parse_expr();
     }
   }
   protected parse_expr() : Expr {
     return this.parse_primary_expr();
   }
}
