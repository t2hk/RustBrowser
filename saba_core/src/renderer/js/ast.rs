use crate::renderer::js::token::JsLexer;
use crate::renderer::js::token::Token;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::iter::Peekable;

/// Javascript の抽象構文木 (AST) を構築するために使うノード列挙型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    ExpressionStatement(Option<Rc<Node>>),
    AdditiveExpression {
        operator: char,
        left: Option<Rc<Node>>,
        right: Option<Rc<Node>>,
    },
    AssignmentExpression {
        operator: char,
        left: Option<Rc<Node>>,
        right: Option<Rc<Node>>,
    },
    MemberExpression {
        object: Option<Rc<Node>>,
        property: Option<Rc<Node>>,
    },
    NumericLiteral(u64),
}

impl Node {
    pub fn new_expression_statement(expression: Option<Rc<Self>>) -> Option<Rc<Self>> {
        Some(Rc::new(Node::ExpressionStatement(expression)))
    }

    pub fn new_additive_expression(
        operator: char,
        left: Option<Rc<Node>>,
        right: Option<Rc<Node>>,
    ) -> Option<Rc<Self>> {
        Some(Rc::new(Node::AdditiveExpression {
            operator,
            left,
            right,
        }))
    }

    pub fn new_assignment_expression(
        operator: char,
        left: Option<Rc<Node>>,
        right: Option<Rc<Node>>,
    ) -> Option<Rc<Self>> {
        Some(Rc::new(Node::AssignmentExpression {
            operator,
            left,
            right,
        }))
    }

    pub fn new_member_expression(
        object: Option<Rc<Self>>,
        property: Option<Rc<Self>>,
    ) -> Option<Rc<Self>> {
        Some(Rc::new(Node::MemberExpression { object, property }))
    }

    pub fn new_numeric_literal(value: u64) -> Option<Rc<Self>> {
        Some(Rc::new(Node::NumericLiteral(value)))
    }
}

/// AST を構築する JsParser 構造体
pub struct JsParser {
    t: Peekable<JsLexer>,
}

impl JsParser {
    pub fn new(t: JsLexer) -> Self {
        Self { t: t.peekable() }
    }

    /// AST を構築する。
    /// BNF の Program を定義する。
    /// Program ::= ( SourceElements )? <EOF>
    pub fn parse_ast(&mut self) -> Program {
        let mut program = Program::new();
        let mut body = Vec::new();

        // ファイルの終端に到達し、ノードを作成できなくなるまで繰り返す。
        loop {
            let node = self.source_element();

            match node {
                Some(n) => body.push(n),
                None => {
                    // ノードを作成できなくなった場合、これまで作成したノードのベクタを body にセットして、今まで構築した AST を返却する。
                    program.set_body(body);
                    return program;
                }
            }
        }
    }

    /// BNF の Statement と ExpressionStatement を解釈する。
    /// Statement ::= ExpressionStatement
    /// ExpressionStatement ::= AssignmentExpression ( ";" )?
    fn statement(&mut self) -> Option<Rc<Node>> {
        let node = Node::new_expression_statement(self.assignment_expression());

        if let Some(Token::Punctuator(c)) = self.t.peek() {
            // ';' を消費する。
            if c == &';' {
                assert!(self.t.next().is_some());
            }
        }
        node
    }

    /// BNF の AssignExpression を解釈する。
    /// AssignmentExpression ::= AdditiveExpression
    fn assignment_expression(&mut self) -> Option<Rc<Node>> {
        self.additive_expression()
    }

    /// BNF の AdditiveExpression を解釈する。
    /// AdditiveExpression ::= LeftHandSideExpression ( AdditiveOperator AssignmentExpression )*
    fn additive_expression(&mut self) -> Option<Rc<Node>> {
        // 足し算や引き算の左辺となるノードを作成する。
        let left = self.left_hand_side_expression();

        let t = match self.t.peek() {
            Some(token) => token.clone(),
            // トークンが存在しない場合、作成したノードをそのまま返す。
            None => return left,
        };

        match t {
            Token::Punctuator(c) => match c {
                // '+' または '-' の記号を消費する。
                '+' | '-' => {
                    assert!(self.t.next().is_some());
                    Node::new_additive_expression(c, left, self.assignment_expression())
                }
                _ => left,
            },
            _ => left,
        }
    }

    /// BNF の LeftHandSideExpression を解釈する。
    /// LeftHandSideExpression ::= MemberExpression
    fn left_hand_side_expression(&mut self) -> Option<Rc<Node>> {
        self.member_expression()
    }

    /// BNF の MemberExpression を解釈する。
    /// MemberExpression ::= PrimaryExpression
    fn member_expression(&mut self) -> Option<Rc<Node>> {
        self.primary_expression()
    }

    /// BNF の PrimaryExpression を解釈する。
    /// PrimaryExpression は配列、変数や関数名、文字や数値リテラルを表す。
    /// ここではまず数値のみとする。
    /// PrimaryExpression ::= Literal
    /// Literal ::= <digit>+
    /// <digit> ::= 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9
    fn primary_expression(&mut self) -> Option<Rc<Node>> {
        let t = match self.t.next() {
            Some(token) => token,
            None => return None,
        };

        match t {
            Token::Number(value) => Node::new_numeric_literal(value),
            _ => None,
        }
    }

    /// BNF の SourceElement を解釈する。
    /// SourceElement ::= Statement
    fn source_element(&mut self) -> Option<Rc<Node>> {
        match self.t.peek() {
            Some(t) => t,
            None => return None,
        };

        self.statement()
    }
}

/// AST のルートノードとなる Program 構造体
/// フィールドに BNF の SourceElements を表す Node のベクタを持つ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    body: Vec<Rc<Node>>,
}

impl Program {
    pub fn new() -> Self {
        Self { body: Vec::new() }
    }

    pub fn set_body(&mut self, body: Vec<Rc<Node>>) {
        self.body = body;
    }

    pub fn body(&self) -> &Vec<Rc<Node>> {
        &self.body
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    /// 空文字のテスト
    #[test]
    fn test_empty() {
        let input = "".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let expected = Program::new();
        assert_eq!(expected, parser.parse_ast());
    }

    /// 1つの数字だけのテスト
    /// Program 構造体の body は Node::ExpressionStatement で囲まれた Node::NumericLiteral を持つはずである。
    #[test]
    fn test_num() {
        let input = "42".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let mut expected = Program::new();
        let mut body = Vec::new();
        body.push(Rc::new(Node::ExpressionStatement(Some(Rc::new(
            Node::NumericLiteral(42),
        )))));
        expected.set_body(body);
        assert_eq!(expected, parser.parse_ast());
    }

    /// 足し算のテスト
    /// 簡単な足し算の場合、Program 構造体の body は Node::ExpressionStatement で囲まれた Node::AdditiveExpression を持つはずである。
    /// また、Node::BinaryExpression は左辺と右辺に数値を表すノードを持つはずである。
    #[test]
    fn test_add_nums() {
        let input = "1 + 2".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let mut expected = Program::new();
        let mut body = Vec::new();
        body.push(Rc::new(Node::ExpressionStatement(Some(Rc::new(
            Node::AdditiveExpression {
                operator: '+',
                left: Some(Rc::new(Node::NumericLiteral(1))),
                right: Some(Rc::new(Node::NumericLiteral(2))),
            },
        )))));
        expected.set_body(body);
        assert_eq!(expected, parser.parse_ast());
    }
}
