namespace AtomicLang
module AST =
  type NodeType =
  | Program
  | Num
  | Error
  | Null
  | Operator
  | BinaryExpr
  | EOP

  [<StructuredFormatDisplay("line: {line}, colmun: {colmun}")>]
  type Expr =
    abstract member Type : NodeType
    abstract member line : int
    abstract member colmun : int

  [<StructuredFormatDisplay("got program => body: {body}, line: {getLine}, colmun: {getColmun}")>]
  type Program(aline : int, acolmun : int)  as self =
    interface Expr with
      member this.line = aline
      member this.colmun = acolmun
      member this.Type = NodeType.Program
    end
    member x.getLine = (self :> Expr).line;
    member x.getColmun = (self :> Expr).colmun;

    member val body : Expr list = [] with get, set



  [<StructuredFormatDisplay("got error => value: {value}, line: {getLine}, colmun: {getColmun}")>]
  type Error(aline : int, acolmun : int, value : string) as self = 
    interface Expr with
      member this.line = aline
      member this.colmun = acolmun

      member this.Type : NodeType = NodeType.Num
    end
    member x.getLine = (self :> Expr).line;
    member x.getColmun = (self :> Expr).colmun;

    member val value : string = value with get, set

  [<StructuredFormatDisplay("got null? => line: {getLine}, colmun: {getColmun}")>]
  type Null(aline : int, acolmun : int) as self = 
    interface Expr with
      member this.line = aline
      member this.colmun = acolmun

      member this.Type : NodeType = NodeType.Null
    end
    member x.getLine = (self :> Expr).line;
    member x.getColmun = (self :> Expr).colmun;
  [<StructuredFormatDisplay("got END => line: {getLine}, colmun: {getColmun}")>]
  type EOP(aline : int, acolmun : int) as self = 
    interface Expr with
      member this.line = aline
      member this.colmun = acolmun

      member this.Type : NodeType = NodeType.EOP
    end
    member x.getLine = (self :> Expr).line;
    member x.getColmun = (self :> Expr).colmun;


  [<StructuredFormatDisplay("got num => value: {value}, line: {getLine}, colmun: {getColmun}")>]
  type Num(aline : int, acolmun : int, value : float) as self = 
    interface Expr with
      member this.line = aline
      member this.colmun = acolmun

      member this.Type : NodeType = NodeType.Num
    end
    member x.getLine = (self :> Expr).line;
    member x.getColmun = (self :> Expr).colmun;

    member val value : float = value with get, set

  [<StructuredFormatDisplay("got operator => value: {value}, line: {getLine}, colmun: {getColmun}")>]
  type operator(aline : int, acolmun : int, value : string) as self= 
    interface Expr with
      member this.line = aline
      member this.colmun = acolmun

      member this.Type : NodeType = NodeType.Operator
    end
    member x.getLine = (self :> Expr).line;
    member x.getColmun = (self :> Expr).colmun;

    member val value : string = value with get, set

  [<StructuredFormatDisplay("left: {left}, right: {right}, op: {operator}, line: {getLine}, colmun: {getColmun}")>]
  type BinaryExpr(aline : int, acolmun : int, left : Expr, right : Expr, operator : operator) as self= 
    interface Expr with
      member this.line = aline
      member this.colmun = acolmun

      member this.Type : NodeType = NodeType.BinaryExpr
    end
    member x.getLine = (self :> Expr).line;
    member x.getColmun = (self :> Expr).colmun;
  
    member val left : Expr = left with get, set
    member val right : Expr = right with get, set
    member val operator : operator = operator with get, set