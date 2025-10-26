use std::collections::HashMap;

use crate::arena::Arena;
use crate::ast::{AstNode, BinOp, UnaryOp, ViraType};

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Array(Vec<Value>),
}

pub struct Interpreter {
    variables: HashMap<String, Value>,
    functions: HashMap<String, AstNode>,
    arena: Arena,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            variables: HashMap::new(),
            functions: HashMap::new(),
            arena: Arena::new(),
        }
    }

    pub fn interpret(&mut self, ast: &[AstNode]) -> Result<(), String> {
        for node in ast {
            self.execute(node)?;
        }
        Ok(())
    }

    fn execute(&mut self, node: &AstNode) -> Result<Value, String> {
        match node {
            AstNode::Literal(val) => Ok(Value::Int(*val)),
            AstNode::FloatLiteral(val) => Ok(Value::Float(*val)),
            AstNode::BoolLiteral(val) => Ok(Value::Bool(*val)),
            AstNode::StringLiteral(s) => Ok(Value::String(s.clone())),
            AstNode::Binary(left, op, right) => {
                let l = self.execute(left)?;
                let r = self.execute(right)?;
                match (l, r, op) {
                    (Value::Int(a), Value::Int(b), BinOp::Add) => Ok(Value::Int(a + b)),
                    (Value::Int(a), Value::Int(b), BinOp::Sub) => Ok(Value::Int(a - b)),
                    (Value::Int(a), Value::Int(b), BinOp::Mul) => Ok(Value::Int(a * b)),
                    (Value::Int(a), Value::Int(b), BinOp::Div) => Ok(Value::Int(a / b)),
                    (Value::Int(a), Value::Int(b), BinOp::Mod) => Ok(Value::Int(a % b)),
                    (Value::Bool(a), Value::Bool(b), BinOp::And) => Ok(Value::Bool(a && b)),
                    (Value::Bool(a), Value::Bool(b), BinOp::Or) => Ok(Value::Bool(a || b)),
                    // Add more, e.g., for float, eq, etc.
                    _ => Err("Type mismatch in binary op.".to_string()),
                }
            }
            AstNode::Unary(op, right) => {
                let r = self.execute(right)?;
                match (op, r) {
                    (UnaryOp::Neg, Value::Int(v)) => Ok(Value::Int(-v)),
                    (UnaryOp::Neg, Value::Float(v)) => Ok(Value::Float(-v)),
                    (UnaryOp::Not, Value::Bool(v)) => Ok(Value::Bool(!v)),
                    _ => Err("Invalid unary op.".to_string()),
                }
            }
            AstNode::VarDecl(name, _, init) => {
                let value = self.execute(init)?;
                self.variables.insert(name.clone(), value);
                Ok(Value::Int(0))
            }
            AstNode::VarRef(name) => self.variables.get(name).cloned().ok_or("Undefined variable.".to_string()),
            AstNode::FuncDecl(name, _, _, body) => {
                self.functions.insert(name.clone(), *body.clone());
                Ok(Value::Int(0))
            }
            AstNode::Call(name, args) => {
                let func = self.functions.get(name).ok_or("Undefined function.")?;
                // Simplified, add param binding
                self.execute(func)
            }
            AstNode::If(cond, then, else_) => {
                if let Value::Bool(true) = self.execute(cond)? {
                    self.execute(then)
                } else if let Some(e) = else_ {
                    self.execute(e)
                } else {
                    Ok(Value::Int(0))
                }
            }
            AstNode::While(cond, body) => {
                while if let Value::Bool(c) = self.execute(cond)? { c } else { false } {
                    self.execute(body)?;
                }
                Ok(Value::Int(0))
            }
            AstNode::For(_, init, cond, incr, body) => {
                self.execute(init)?;
                while if let Value::Bool(c) = self.execute(cond)? { c } else { false } {
                    self.execute(body)?;
                    self.execute(incr)?;
                }
                Ok(Value::Int(0))
            }
            AstNode::Return(expr) => {
                if let Some(e) = expr {
                    self.execute(e)
                } else {
                    Ok(Value::Int(0))
                }
            }
            AstNode::Block(stmts) => {
                let mut result = Value::Int(0);
                for stmt in stmts {
                    result = self.execute(stmt)?;
                }
                Ok(result)
            }
            AstNode::Write(expr) => {
                let value = self.execute(expr)?;
                println!("{:?}", value);
                Ok(Value::Int(0))
            }
            AstNode::ArrayLiteral(elems) => {
                let mut arr = Vec::new();
                for elem in elems {
                    arr.push(self.execute(elem)?);
                }
                Ok(Value::Array(arr))
            }
            AstNode::Index(arr, idx) => {
                let a = self.execute(arr)?;
                let i = self.execute(idx)?;
                if let Value::Array(vec) = a {
                    if let Value::Int(index) = i {
                        vec.get(index as usize).cloned().ok_or("Index out of bounds.".to_string())
                    } else {
                        Err("Index must be int.".to_string())
                    }
                } else {
                    Err("Cannot index non-array.".to_string())
                }
            }
        }
    }
}
