use crate::renderer::js::token::JsLexer;
use crate::renderer::js::token::Token;
use alloc::rc::Rc;
use alloc::string::String;
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
    /// var から始まる宣言を表す。
    VariableDeclaration {
        declarations: Vec<Option<Rc<Node>>>,
    },
    /// 変数と初期化式を表す。
    VariableDeclarator {
        id: Option<Rc<Node>>,
        init: Option<Rc<Node>>,
    },
    /// 変数をあわわス。
    Identifier(String),
    /// 文字列を表す。
    StringLiteral(String),

    /// {} で囲まれるブロックを表す。
    BlockStatement {
        body: Vec<Option<Rc<Node>>>,
    },
    /// return の予約語から始まる文を表す。
    ReturnStatement {
        argument: Option<Rc<Node>>,
    },
    /// function の予約語から始まる文を表す。
    FunctionDeclaration {
        id: Option<Rc<Node>>,
        params: Vec<Option<Rc<Node>>>,
        body: Option<Rc<Node>>,
    },
    /// 関数呼び出しを表す。
    CallExpression {
        callee: Option<Rc<Node>>,
        arguments: Vec<Option<Rc<Node>>>,
    },
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

    pub fn new_variable_declarator(
        id: Option<Rc<Self>>,
        init: Option<Rc<Self>>,
    ) -> Option<Rc<Self>> {
        Some(Rc::new(Node::VariableDeclarator { id, init }))
    }

    pub fn new_variable_declaration(declarations: Vec<Option<Rc<Self>>>) -> Option<Rc<Self>> {
        Some(Rc::new(Node::VariableDeclaration { declarations }))
    }

    pub fn new_identifier(name: String) -> Option<Rc<Self>> {
        Some(Rc::new(Node::Identifier(name)))
    }

    pub fn new_string_literal(value: String) -> Option<Rc<Self>> {
        Some(Rc::new(Node::StringLiteral(value)))
    }

    pub fn new_block_statement(body: Vec<Option<Rc<Self>>>) -> Option<Rc<Self>> {
        Some(Rc::new(Node::BlockStatement { body }))
    }

    pub fn new_return_statement(argument: Option<Rc<Self>>) -> Option<Rc<Self>> {
        Some(Rc::new(Node::ReturnStatement { argument }))
    }

    pub fn new_function_declaration(
        id: Option<Rc<Self>>,
        params: Vec<Option<Rc<Self>>>,
        body: Option<Rc<Self>>,
    ) -> Option<Rc<Self>> {
        Some(Rc::new(Node::FunctionDeclaration { id, params, body }))
    }

    pub fn new_call_expression(
        callee: Option<Rc<Self>>,
        arguments: Vec<Option<Rc<Self>>>,
    ) -> Option<Rc<Self>> {
        Some(Rc::new(Node::CallExpression { callee, arguments }))
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

    /// BNF の Statement として ExpressionStatement、VariableStatement、ReturnStatement を解釈する。
    /// Statement ::= ExpressionStatement | VariableStatement | ReturnStatemet
    /// ExpressionStatement ::= AssignmentExpression ( ";" )?
    fn statement(&mut self) -> Option<Rc<Node>> {
        let t = match self.t.peek() {
            Some(t) => t,
            None => return None,
        };

        let node = match t {
            Token::Keyword(keyword) => {
                if keyword == "var" {
                    // "var" の予約語を消費する。
                    assert!(self.t.next().is_some());
                    self.variable_declaration()
                } else if keyword == "return" {
                    // "return" の予約語を消費する。
                    assert!(self.t.next().is_some());
                    Node::new_return_statement(self.assignment_expression())
                } else {
                    None
                }
            }
            _ => Node::new_expression_statement(self.assignment_expression()),
        };

        // if let Some(Token::Punctuator(c)) = self.t.peek() {
        if let Some(t) = self.t.peek() {
            if let Token::Punctuator(c) = t {
                // ';' を消費する。
                if c == &';' {
                    assert!(self.t.next().is_some());
                }
            }
        }
        node
    }

    /// BNF の AssignExpression を解釈する。
    /// AssignmentExpression ::= AdditiveExpression ("=" AdditiveExpression )*
    fn assignment_expression(&mut self) -> Option<Rc<Node>> {
        let expr = self.additive_expression();

        let t = match self.t.peek() {
            Some(token) => token,
            None => return expr,
        };

        match t {
            Token::Punctuator('=') => {
                // '=' を消費する。
                assert!(self.t.next().is_some());
                Node::new_assignment_expression('=', expr, self.assignment_expression())
            }
            _ => expr,
        }
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
    /// LeftHandSideExpression ::= CallExpression | MemberExpression
    fn left_hand_side_expression(&mut self) -> Option<Rc<Node>> {
        let expr = self.member_expression();

        let t = match self.t.peek() {
            Some(token) => token,
            None => return expr,
        };

        match t {
            Token::Punctuator(c) => {
                if c == &'(' {
                    // '(' を消費する。
                    assert!(self.t.next().is_some());
                    return Node::new_call_expression(expr, self.arguments());
                }
                expr
            }
            _ => expr,
        }
    }

    /// BNF の MemberExpression を解釈する。
    /// MemberExpression ::= PrimaryExpression ( "." Identifier )*
    fn member_expression(&mut self) -> Option<Rc<Node>> {
        let expr = self.primary_expression();

        let t = match self.t.peek() {
            Some(token) => token,
            None => return expr,
        };

        match t {
            Token::Punctuator(c) => {
                if c == &'.' {
                    // '.' を消費する。
                    assert!(self.t.next().is_some());
                    return Node::new_member_expression(expr, self.identifier());
                }
                expr
            }
            _ => expr,
        }
    }

    /// BNF の PrimaryExpression を解釈する。
    /// PrimaryExpression は配列、変数や関数名、文字や数値リテラルを表す。
    /// PrimaryExpression ::= Identifier | Literal
    /// Literal ::= <digit>+
    /// <digit> ::= 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9
    fn primary_expression(&mut self) -> Option<Rc<Node>> {
        let t = match self.t.next() {
            Some(token) => token,
            None => return None,
        };

        match t {
            Token::Identifier(value) => Node::new_identifier(value),
            Token::StringLiteral(value) => Node::new_string_literal(value),
            Token::Number(value) => Node::new_numeric_literal(value),
            _ => None,
        }
    }

    /// BNF の SourceElement を解釈する。
    /// SourceElement ::= FunctionDeclaration | Statement
    fn source_element(&mut self) -> Option<Rc<Node>> {
        let t = match self.t.peek() {
            Some(t) => t,
            None => return None,
        };

        match t {
            Token::Keyword(keyword) => {
                // function キーワードを消費する。
                if keyword == "function" {
                    assert!(self.t.next().is_some());
                    self.function_declaration()
                } else {
                    self.statement()
                }
            }
            _ => self.statement(),
        }
    }

    /// VariableDeclaration の解釈
    /// VariableDeclaration は変数とその初期化式によって成立する。以下の BNF を実装する。
    /// VariableDeclaration ::= Identifier ( Initializer )?
    fn variable_declaration(&mut self) -> Option<Rc<Node>> {
        let ident = self.identifier();
        let declarator = Node::new_variable_declarator(ident, self.initializer());
        let mut declarations = Vec::new();
        declarations.push(declarator);

        Node::new_variable_declaration(declarations)
    }

    /// Identifier の解釈
    /// Identifier は変数を表す。BNF 以下の通り。
    /// Identifier ::= <identifier name>
    /// <Identifier name> ::= (& | _ | a-z | A-Z) (& | a-z | A-Z)*
    fn identifier(&mut self) -> Option<Rc<Node>> {
        let t = match self.t.next() {
            Some(token) => token,
            None => return None,
        };

        match t {
            Token::Identifier(name) => Node::new_identifier(name),
            _ => None,
        }
    }

    /// Initializer の解釈
    /// Initializer はイコール (=) と初期値を表す AssignmentExpression によって置き換え可能である。
    /// BNF は以下の通り。
    /// Initializer ::= "=" AssignmentExpression
    fn initializer(&mut self) -> Option<Rc<Node>> {
        let t = match self.t.next() {
            Some(token) => token,
            None => return None,
        };

        match t {
            Token::Punctuator(c) => match c {
                '=' => self.assignment_expression(),
                _ => None,
            },
            _ => None,
        }
    }

    /// FunctionDeclaration の解釈
    /// BNF は以下の通り。
    /// FunctionDeclaration ::= "function" Identifier ( "(" ( FormalParameterList )? ")" ) FunctionBody
    fn function_declaration(&mut self) -> Option<Rc<Node>> {
        let id = self.identifier();
        let params = self.parameter_list();
        Node::new_function_declaration(id, params, self.function_body())
    }

    /// FormalParameterList の解釈
    /// 丸括弧 ( を消費し、閉じ括弧 ) に到達するまでカンマ , で区切られた変数をパラメータに追加する。
    /// BNF は以下の通り。
    /// FormalParameterList ::= Identifier ( "." Identifier )*
    fn parameter_list(&mut self) -> Vec<Option<Rc<Node>>> {
        let mut params = Vec::new();

        // '(' を消費する。もし次のトークンが '(' ではない場合、エラーになる。
        match self.t.next() {
            Some(t) => match t {
                Token::Punctuator(c) => assert!(c == '('),
                _ => unimplemented!("function should have '(' but got {:?}", t),
            },
            None => unimplemented!("function should have '(' but got None"),
        }

        loop {
            // ')' に到達するまで、params に仮引数となる変数を追加する。
            match self.t.peek() {
                Some(t) => match t {
                    Token::Punctuator(c) => {
                        if c == &')' {
                            // ')' を消費する。
                            assert!(self.t.next().is_some());
                            return params;
                        }
                        if c == &',' {
                            // ',' を消費する。
                            assert!(self.t.next().is_some());
                        }
                    }
                    _ => {
                        params.push(self.identifier());
                    }
                },
                None => return params,
            }
        }
    }

    /// FunctionBody の解釈
    /// 関数のボディを表す FunctionBody を解釈する。
    /// 開き波括弧 { を消費し、閉じ波括弧 } が現れるまで関数の文として解釈する。
    /// BNFは以下の通り。
    /// FunctionBody ::= "{" ( SourceElements )? "}"
    fn function_body(&mut self) -> Option<Rc<Node>> {
        match self.t.next() {
            Some(t) => match t {
                Token::Punctuator(c) => assert!(c == '{'),
                _ => unimplemented!("function should have open curly blacket but got {:?}", t),
            },
            None => unimplemented!("function should have open curly blacket but got None"),
        }

        let mut body = Vec::new();

        loop {
            // } に到達するまで、関数内のコードとして解釈する。
            match self.t.peek() {
                Some(t) => match t {
                    Token::Punctuator(c) => {
                        // } を消費し、BlockStatement ノードを返す。
                        if c == &'}' {
                            assert!(self.t.next().is_some());
                            return Node::new_block_statement(body);
                        }
                    }
                    _ => {}
                },
                None => {}
            }
            body.push(self.source_element());
        }
    }

    /// Arguments の解釈
    /// Arguments は関数呼び出しに必要な引数を表す。
    /// 閉じ丸括弧 ) が現れるまで、解釈した値を arguments 変数に追加する。閉じ丸括弧 ) が現れたら、今まで解釈した値のベクタを返す。
    /// BNF は以下の通り。
    /// Arguments ::= "(" ( ArgumentList )? ")"
    /// ArgumentList ::= AssiginmentExpression ( "," AssignmentExpression )*
    fn arguments(&mut self) -> Vec<Option<Rc<Node>>> {
        let mut arguments = Vec::new();

        loop {
            // ) に到達するまで、解釈した値を arguments ベクタに追加する。
            match self.t.peek() {
                Some(t) => match t {
                    Token::Punctuator(c) => {
                        // ) を消費する。
                        if c == &')' {
                            assert!(self.t.next().is_some());
                            return arguments;
                        }
                        // , を消費する。
                        if c == &',' {
                            assert!(self.t.next().is_some());
                        }
                    }
                    _ => arguments.push(self.assignment_expression()),
                },
                None => return arguments,
            }
        }
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

    /// 変数定義のテスト
    /// var foo="bar"; を入力とするテスト。
    /// Program の body には変数定義文である VariableDeclaration が存在し、
    /// 変数名が foo、初期値が bar であることを確認する。
    #[test]
    fn test_assign_variable() {
        let input = "var foo=\"bar\";".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let mut expected = Program::new();
        let mut body = Vec::new();
        body.push(Rc::new(Node::VariableDeclaration {
            declarations: [Some(Rc::new(Node::VariableDeclarator {
                id: Some(Rc::new(Node::Identifier("foo".to_string()))),
                init: Some(Rc::new(Node::StringLiteral("bar".to_string()))),
            }))]
            .to_vec(),
        }));
        expected.set_body(body);
        assert_eq!(expected, parser.parse_ast());
    }

    /// 変数呼び出すのテスト
    /// var foo = 42; var result = foo+1; を入力とするテスト
    /// Program の body には2つの文が存在するため長さが 2 であることを確認する。
    /// どちらの要素も VariableDeclaration の文である。
    #[test]
    fn test_add_variable_and_num() {
        let input = "var foo = 42; var result = foo + 1;".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let mut expected = Program::new();
        let mut body = Vec::new();
        body.push(Rc::new(Node::VariableDeclaration {
            declarations: [Some(Rc::new(Node::VariableDeclarator {
                id: Some(Rc::new(Node::Identifier("foo".to_string()))),
                init: Some(Rc::new(Node::NumericLiteral(42))),
            }))]
            .to_vec(),
        }));

        body.push(Rc::new(Node::VariableDeclaration {
            declarations: [Some(Rc::new(Node::VariableDeclarator {
                id: Some(Rc::new(Node::Identifier("result".to_string()))),
                init: Some(Rc::new(Node::AdditiveExpression {
                    operator: '+',
                    left: Some(Rc::new(Node::Identifier("foo".to_string()))),
                    right: Some(Rc::new(Node::NumericLiteral(1))),
                })),
            }))]
            .to_vec(),
        }));
        expected.set_body(body);
        assert_eq!(expected, parser.parse_ast());
    }

    /// 関数定義のテスト
    /// foo 関数内で return 文により数値を返却している。
    #[test]
    fn test_define_function() {
        let input = "function foo() { return 42; }".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let mut expected = Program::new();
        let mut body = Vec::new();
        body.push(Rc::new(Node::FunctionDeclaration {
            id: Some(Rc::new(Node::Identifier("foo".to_string()))),
            params: [].to_vec(),
            body: Some(Rc::new(Node::BlockStatement {
                body: [Some(Rc::new(Node::ReturnStatement {
                    argument: Some(Rc::new(Node::NumericLiteral(42))),
                }))]
                .to_vec(),
            })),
        }));
        expected.set_body(body);
        assert_eq!(expected, parser.parse_ast());
    }

    /// 引数付き関数定義のテスト
    /// 引数を持つ関数の定義のテストを行う。
    /// 関数 foo は 2つの引数 a と b を持ち、それらの加算結果を返す。
    #[test]
    fn test_define_function_with_args() {
        let input = "function foo (a, b) { return a + b; }".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let mut expected = Program::new();
        let mut body = Vec::new();
        body.push(Rc::new(Node::FunctionDeclaration {
            id: Some(Rc::new(Node::Identifier("foo".to_string()))),
            params: [
                Some(Rc::new(Node::Identifier("a".to_string()))),
                Some(Rc::new(Node::Identifier("b".to_string()))),
            ]
            .to_vec(),
            body: Some(Rc::new(Node::BlockStatement {
                body: [Some(Rc::new(Node::ReturnStatement {
                    argument: Some(Rc::new(Node::AdditiveExpression {
                        operator: '+',
                        left: Some(Rc::new(Node::Identifier("a".to_string()))),
                        right: Some(Rc::new(Node::Identifier("b".to_string()))),
                    })),
                }))]
                .to_vec(),
            })),
        }));
        expected.set_body(body);
        assert_eq!(expected, parser.parse_ast());
    }
}
