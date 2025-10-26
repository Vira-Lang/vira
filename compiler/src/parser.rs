use crate::ast::{AstNode, BinOp, UnaryOp, ViraType};
use crate::tokenizer::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<AstNode>, String> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.statement()?);
        }
        Ok(statements)
    }

    fn statement(&mut self) -> Result<AstNode, String> {
        if self.match_token(TokenType::Func) {
            self.func_decl()
        } else if self.match_token(TokenType::Let) {
            self.var_decl()
        } else if self.match_token(TokenType::If) {
            self.if_stmt()
        } else if self.match_token(TokenType::While) {
            self.while_stmt()
        } else if self.match_token(TokenType::For) {
            self.for_stmt()
        } else if self.match_token(TokenType::Return) {
            self.return_stmt()
        } else if self.match_token(TokenType::Write) {
            self.write_stmt()
        } else if self.match_token(TokenType::LeftBracket) || self.match_token(TokenType::LeftBrace) {
            self.block()
        } else {
            self.expression_stmt()
        }
    }

    fn func_decl(&mut self) -> Result<AstNode, String> {
        let name = self.consume(TokenType::Identifier, "Expect function name.")?.lexeme;
        self.consume(TokenType::LeftParen, "Expect '(' after name.")?;
        let mut params = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                let param_name = self.consume(TokenType::Identifier, "Expect param name.")?.lexeme;
                self.consume(TokenType::Colon, "Expect ':' after param name.")?;
                let param_type = self.parse_type()?;
                params.push((param_name, param_type));
                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after params.")?;
        if self.match_token(TokenType::Arrow) {
            let return_type = self.parse_type()?;
            let body = self.statement()?;
            Ok(AstNode::FuncDecl(name, params, return_type, Box::new(body)))
        } else {
            Err("Missing '->' in function declaration.".to_string())
        }
    }

    fn var_decl(&mut self) -> Result<AstNode, String> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?.lexeme;
        let mut typ = ViraType::Int;
        if self.match_token(TokenType::Colon) {
            typ = self.parse_type()?;
        }
        self.consume(TokenType::Equals, "Expect '=' after variable.")?;
        let init = self.expression()?;
        Ok(AstNode::VarDecl(name, typ, Box::new(init)))
    }

    fn if_stmt(&mut self) -> Result<AstNode, String> {
        let cond = self.expression()?;
        let then = self.statement()?;
        let else_branch = if self.match_token(TokenType::Else) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        Ok(AstNode::If(Box::new(cond), Box::new(then), else_branch))
    }

    fn while_stmt(&mut self) -> Result<AstNode, String> {
        let cond = self.expression()?;
        let body = self.statement()?;
        Ok(AstNode::While(Box::new(cond), Box::new(body)))
    }

    fn for_stmt(&mut self) -> Result<AstNode, String> {
        let init = self.statement()?;
        let cond = self.expression()?;
        let incr = self.expression()?;
        let body = self.statement()?;
        Ok(AstNode::For("".to_string(), Box::new(init), Box::new(cond), Box::new(incr), Box::new(body)))
    }

    fn return_stmt(&mut self) -> Result<AstNode, String> {
        let expr = if !self.check(TokenType::RightBracket) && !self.check(TokenType::RightBrace) {
            Some(Box::new(self.expression()?))
        } else {
            None
        };
        Ok(AstNode::Return(expr))
    }

    fn write_stmt(&mut self) -> Result<AstNode, String> {
        let expr = self.expression()?;
        Ok(AstNode::Write(Box::new(expr)))
    }

    fn block(&mut self) -> Result<AstNode, String> {
        let mut statements = Vec::new();
        while !self.check(TokenType::RightBracket) && !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.statement()?);
        }
        if self.check(TokenType::RightBracket) {
            self.consume(TokenType::RightBracket, "Expect ']' after block.")?;
        } else {
            self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        }
        Ok(AstNode::Block(statements))
    }

    fn expression_stmt(&mut self) -> Result<AstNode, String> {
        self.expression()
    }

    fn expression(&mut self) -> Result<AstNode, String> {
        self.logical_or()
    }

    fn logical_or(&mut self) -> Result<AstNode, String> {
        let mut expr = self.logical_and()?;
        while self.match_token(TokenType::Or) {
            let right = self.logical_and()?;
            expr = AstNode::Binary(Box::new(expr), BinOp::Or, Box::new(right));
        }
        Ok(expr)
    }

    fn logical_and(&mut self) -> Result<AstNode, String> {
        let mut expr = self.equality()?;
        while self.match_token(TokenType::And) {
            let right = self.equality()?;
            expr = AstNode::Binary(Box::new(expr), BinOp::And, Box::new(right));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<AstNode, String> {
        let mut expr = self.comparison()?;
        while self.match_token(TokenType::EqualEqual) || self.match_token(TokenType::BangEqual) {
            let op = if self.previous().typ == TokenType::EqualEqual { BinOp::Eq } else { BinOp::Neq };
            let right = self.comparison()?;
            expr = AstNode::Binary(Box::new(expr), op, Box::new(right));
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<AstNode, String> {
        let mut expr = self.term()?;
        while self.match_token(TokenType::Less) || self.match_token(TokenType::Greater) || self.match_token(TokenType::LessEqual) || self.match_token(TokenType::GreaterEqual) {
            let op = match self.previous().typ {
                TokenType::Less => BinOp::Lt,
                TokenType::Greater => BinOp::Gt,
                TokenType::LessEqual => BinOp::Le,
                TokenType::GreaterEqual => BinOp::Ge,
                _ => unreachable!(),
            };
            let right = self.term()?;
            expr = AstNode::Binary(Box::new(expr), op, Box::new(right));
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<AstNode, String> {
        let mut expr = self.factor()?;
        while self.match_token(TokenType::Minus) || self.match_token(TokenType::Plus) {
            let op = if self.previous().typ == TokenType::Plus { BinOp::Add } else { BinOp::Sub };
            let right = self.factor()?;
            expr = AstNode::Binary(Box::new(expr), op, Box::new(right));
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<AstNode, String> {
        let mut expr = self.unary()?;
        while self.match_token(TokenType::Star) || self.match_token(TokenType::Slash) || self.match_token(TokenType::Mod) {
            let op = match self.previous().typ {
                TokenType::Star => BinOp::Mul,
                TokenType::Slash => BinOp::Div,
                TokenType::Mod => BinOp::Mod,
                _ => unreachable!(),
            };
            let right = self.unary()?;
            expr = AstNode::Binary(Box::new(expr), op, Box::new(right));
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<AstNode, String> {
        if self.match_token(TokenType::Minus) || self.match_token(TokenType::Bang) {
            let op = if self.previous().typ == TokenType::Minus { UnaryOp::Neg } else { UnaryOp::Not };
            let right = self.unary()?;
            Ok(AstNode::Unary(op, Box::new(right)))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<AstNode, String> {
        if self.match_token(TokenType::Number) {
            let value = self.previous().lexeme.parse::<i64>().map_err(|_| "Invalid number.")?;
            Ok(AstNode::Literal(value))
        } else if self.match_token(TokenType::Float) {
            let value = self.previous().lexeme.parse::<f64>().map_err(|_| "Invalid float.")?;
            Ok(AstNode::FloatLiteral(value))
        } else if self.match_token(TokenType::True) {
            Ok(AstNode::BoolLiteral(true))
        } else if self.match_token(TokenType::False) {
            Ok(AstNode::BoolLiteral(false))
        } else if self.match_token(TokenType::String) {
            Ok(AstNode::StringLiteral(self.previous().lexeme))
        } else if self.match_token(TokenType::Identifier) {
            let name = self.previous().lexeme;
            if self.match_token(TokenType::LeftParen) {
                let mut args = Vec::new();
                if !self.check(TokenType::RightParen) {
                    loop {
                        args.push(self.expression()?);
                        if !self.match_token(TokenType::Comma) {
                            break;
                        }
                    }
                }
                self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;
                Ok(AstNode::Call(name, args))
            } else {
                Ok(AstNode::VarRef(name))
            }
        } else if self.match_token(TokenType::LeftBracket) {
            let mut elements = Vec::new();
            if !self.check(TokenType::RightBracket) {
                loop {
                    elements.push(self.expression()?);
                    if !self.match_token(TokenType::Comma) {
                        break;
                    }
                }
            }
            self.consume(TokenType::RightBracket, "Expect ']' after array.")?;
            Ok(AstNode::ArrayLiteral(elements))
        } else if self.match_token(TokenType::LeftParen) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            Ok(expr)
        } else {
            Err(format!("Unexpected token: {:?}", self.peek()))
        }
    }

    fn parse_type(&mut self) -> Result<ViraType, String> {
        let typ = self.consume(TokenType::Identifier, "Expect type.")?.lexeme;
        match typ.as_str() {
            "int" => Ok(ViraType::Int),
            "float" => Ok(ViraType::Float),
            "bool" => Ok(ViraType::Bool),
            "string" => Ok(ViraType::String),
            "array" => {
                self.consume(TokenType::Less, "Expect '<' for array type.")?;
                let inner = self.parse_type()?;
                self.consume(TokenType::Greater, "Expect '>' for array type.")?;
                Ok(ViraType::Array(Box::new(inner)))
            }
            _ => Err("Unknown type.".to_string()),
        }
    }

    fn consume(&mut self, typ: TokenType, msg: &str) -> Result<Token, String> {
        if self.check(typ) {
            Ok(self.advance())
        } else {
            Err(msg.to_string())
        }
    }

    fn match_token(&mut self, typ: TokenType) -> bool {
        if self.check(typ) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, typ: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().typ == typ
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn is_at_end(&self) -> bool {
        self.peek().typ == TokenType::Eof
    }
}
