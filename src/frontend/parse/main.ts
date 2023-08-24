
import { Stmt, Expr} from "../AST/stmts.ts";
import { Ion, Type } from "../Ion.ts";

export class ParserMain {
   protected ions: Array<Ion>;
   protected line: number = 1;
   protected colmun: number = 1;
   

   public constructor(ions: Ion[]) {
      this.ions = ions;
   }

   protected getTypeName(T: Type) : string{
      let name: string = Type[T];

      if(name.includes("_kw")) {
         return name.replace("_kw", " keyword");
      }
      else if(name.includes("_type")) {
         return name.replace("_type", ""); 
      }
      else {
         return name;
      }
   }
   protected error(msg: string, code: string = "AT000") {
      console.log(`%cParser Error:${msg}\nat => line: ${this.line}, colmun:${this.colmun}\ngot => value:${this.at().value}, type:${this.getTypeName(this.at().type)}, ErrorCode:${code}`, 'color: crimson; background-color: gold');

   }
   protected take() : Ion | undefined {
      if(this.ions.length <= 0) {
         return {
            value: "END",
            type: Type.EOF,
            line: this.line,
            colmun: this.colmun
         } as Ion;
      }
      else {
         this.Update();
         return this.ions.shift();
      }
   }

   protected at() : Ion {
      if(this.ions.length <= 0) {
         return {
            value: "END",
            type: Type.EOF,
            line: this.line,
            colmun: this.colmun
         } as Ion;
      }
      else {
         this.Update();
         return this.ions[0];
      }
   }
   protected Update() {
     this.line = this.ions[0].line;
     this.colmun = this.ions[0].colmun;
   }
   
   protected notEOF(): boolean { 
      return this.at().type != Type.EOF;
   }

   // we just need a code that survives complie time here

   protected parse_stmt() : Stmt {
      return {} as Stmt;
   }

   protected parse_expr() : Expr {
      return {} as Expr;
   }
}