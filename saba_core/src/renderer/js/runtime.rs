use crate::renderer::dom::api::get_element_by_id;
use crate::renderer::dom::node::Node as DomNode;
use crate::renderer::dom::node::NodeKind as DomNodeKind;
use crate::renderer::js::ast::Node;
use crate::renderer::js::ast::Program;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::cell::RefCell;
use core::fmt::Display;
use core::fmt::Formatter;
use core::ops::Add;
use core::ops::Sub;

/// 変数の名前と値のタプルを表す。
type VariableMap = Vec<(String, Option<RuntimeValue>)>;

/// JavaScript の変数のスコープ管理を行うための構造体
/// https://262.ecma-international.org/#sec-environment-records
#[derive(Debug, Clone)]
pub struct Environment {
    variables: VariableMap,                  // 変数の名前と値を表す。
    outer: Option<Rc<RefCell<Environment>>>, // 外部のスコープを表す。内側のスコープからはアクセスできる。
}

impl Environment {
    fn new(outer: Option<Rc<RefCell<Environment>>>) -> Self {
        Self {
            variables: VariableMap::new(),
            outer,
        }
    }

    /// 変数の取得
    /// Environemnt 構造体に保存してる変数について、name と一致する変数の値を返す。
    /// まず variables をチェックし、ローカル変数の中で一致するものがあるかチェックする。
    /// 見つからない場合、外部のスコープの outer をチェックする。
    pub fn get_variable(&self, name: String) -> Option<RuntimeValue> {
        for variable in &self.variables {
            if variable.0 == name {
                return variable.1.clone();
            }
        }

        if let Some(env) = &self.outer {
            env.borrow_mut().get_variable(name)
        } else {
            None
        }
    }

    /// 変数の追加
    /// 現在のスコープに新しい変数を追加する。
    fn add_variable(&mut self, name: String, value: Option<RuntimeValue>) {
        self.variables.push((name, value));
    }

    /// 変数の更新
    /// 現在のスコープに存在する変数の値を更新する。
    fn update_variable(&mut self, name: String, value: Option<RuntimeValue>) {
        for i in 0..self.variables.len() {
            // 変数を見つけた場合、現在の変数を削除し、更新後の変数を追加する。
            if self.variables[i].0 == name {
                self.variables.remove(i);
                self.variables.push((name, value));
                return;
            }
        }
    }
}

/// Function 構造体
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    id: String,
    params: Vec<Option<Rc<Node>>>,
    body: Option<Rc<Node>>,
}
impl Function {
    fn new(id: String, params: Vec<Option<Rc<Node>>>, body: Option<Rc<Node>>) -> Self {
        Self { id, params, body }
    }
}

/// ランタイム用の構造体
#[derive(Debug, Clone)]
pub struct JsRuntime {
    dom_root: Rc<RefCell<DomNode>>,
    env: Rc<RefCell<Environment>>,
    functions: Vec<Function>,
}

impl JsRuntime {
    pub fn new(dom_root: Rc<RefCell<DomNode>>) -> Self {
        Self {
            dom_root,
            functions: Vec::new(),
            env: Rc::new(RefCell::new(Environment::new(None))),
        }
    }

    /// program の body に含まれるノードを eval メソッドによってすべて評価する。
    pub fn execute(&mut self, program: &Program) {
        for node in program.body() {
            self.eval(&Some(node.clone()), self.env.clone());
        }
    }

    /// ノードの種類に応じて処理する。
    fn eval(
        &mut self,
        node: &Option<Rc<Node>>,
        env: Rc<RefCell<Environment>>,
    ) -> Option<RuntimeValue> {
        let node = match node {
            Some(n) => n,
            None => return None,
        };

        match node.borrow() {
            // ExpressionStatement の場合、再帰呼び出し
            Node::ExpressionStatement(expr) => return self.eval(&expr, env.clone()),
            // 加算または減算
            Node::AdditiveExpression {
                operator,
                left,
                right,
            } => {
                let left_value = match self.eval(&left, env.clone()) {
                    Some(value) => value,
                    None => return None,
                };
                let right_value = match self.eval(&right, env.clone()) {
                    Some(value) => value,
                    None => return None,
                };

                if operator == &'+' {
                    Some(left_value + right_value)
                } else if operator == &'-' {
                    Some(left_value - right_value)
                } else {
                    None
                }
            }
            Node::AssignmentExpression {
                operator,
                left,
                right,
            } => {
                if operator != &'=' {
                    return None;
                }
                // 変数の再割り当て
                if let Some(node) = left {
                    if let Node::Identifier(id) = node.borrow() {
                        let new_value = self.eval(right, env.clone());
                        env.borrow_mut().update_variable(id.to_string(), new_value);
                        return None;
                    }
                }
                None
            }
            Node::MemberExpression { object, property } => {
                let object_value = match self.eval(object, env.clone()) {
                    Some(value) => value,
                    None => return None,
                };
                let property_value = match self.eval(property, env.clone()) {
                    Some(value) => value,
                    None => return Some(object_value),
                };

                // document.getElementById は "document.getElementById" という1つの文字列として扱う。
                // このメソッドの呼び出しは、"document.getElementById" という名前の関数の呼び出しとなる。
                return Some(
                    object_value + RuntimeValue::StringLiteral(".".to_string()) + property_value,
                );
            }
            Node::NumericLiteral(value) => Some(RuntimeValue::Number(*value)),
            Node::VariableDeclaration { declarations } => {
                for declaration in declarations {
                    self.eval(&declaration, env.clone());
                }
                None
            }
            Node::VariableDeclarator { id, init } => {
                if let Some(node) = id {
                    if let Node::Identifier(id) = node.borrow() {
                        let init = self.eval(&init, env.clone());
                        env.borrow_mut().add_variable(id.to_string(), init);
                    }
                }
                None
            }
            Node::Identifier(name) => match env.borrow_mut().get_variable(name.to_string()) {
                Some(v) => Some(v),
                None => Some(RuntimeValue::StringLiteral(name.to_string())),
            },
            Node::StringLiteral(value) => Some(RuntimeValue::StringLiteral(value.to_string())),
            Node::BlockStatement { body } => {
                // 波括弧で囲われた BlockStatement ノードを処理する。
                let mut result: Option<RuntimeValue> = None;
                for stmt in body {
                    result = self.eval(&stmt, env.clone());
                }
                result
            }
            Node::ReturnStatement { argument } => {
                // ReturnStatement ノードは戻り値が argument なので eval で解釈する。
                return self.eval(&argument, env.clone());
            }
            Node::FunctionDeclaration { id, params, body } => {
                // FunctionDeclaration ノードで関数名が文字列の場合、今迄に定義した関数を保持する functions フィールドに Function 構造体を追加する。
                if let Some(RuntimeValue::StringLiteral(id)) = self.eval(&id, env.clone()) {
                    let cloned_body = match body {
                        Some(b) => Some(b.clone()),
                        None => None,
                    };
                    self.functions
                        .push(Function::new(id, params.to_vec(), cloned_body));
                };
                None
            }
            // CallExpression ノードの場合
            Node::CallExpression { callee, arguments } => {
                // 新しいスコープを作成する。
                // コンストラクタに現在のスコープである env を渡すことで、現在のスコープが新しく作成するスコープの外側のスコープになる。
                let new_env = Rc::new(RefCell::new(Environment::new(Some(env))));

                // callee ノードを解釈し、その関数名と一致する関数を探す。
                let callee_value = match self.eval(callee, new_env.clone()) {
                    Some(value) => value,
                    None => return None,
                };

                // 既に定義されている関数を探す。
                // FunctionDeclaration ノードの解釈時に追加した関数の中から探す。
                // もし関数名に一致する関数が見つからない場合、パニックマクロを呼ぶ。
                let function = {
                    let mut f: Option<Function> = None;

                    for func in &self.functions {
                        if callee_value == RuntimeValue::StringLiteral(func.id.to_string()) {
                            f = Some(func.clone());
                        }
                    }
                    match f {
                        Some(f) => f,
                        None => panic!("function {:?} doesn't exist", callee),
                    }
                };

                // 関数呼び出し時に渡される引数を新しく作成したスコープのローカル変数として割り当てる。
                assert!(arguments.len() == function.params.len());
                for (i, item) in arguments.iter().enumerate() {
                    if let Some(RuntimeValue::StringLiteral(name)) =
                        self.eval(&function.params[i], new_env.clone())
                    {
                        new_env
                            .borrow_mut()
                            .add_variable(name, self.eval(item, new_env.clone()));
                    }
                }
                // 関数の中身を新しいスコープとともに eval で解釈する。
                self.eval(&function.body.clone(), new_env.clone())
            } //_ => todo!(),
        }
    }

    /// ブラウザがサポートするブラウザ API を呼ぶ。
    /// 引数:
    ///   * 関数名 (func)
    ///   * 引数 (arguments)
    ///   * スコープ (env)
    /// 戻り値:
    ///   * (bool, Option<RuntimeValue>) タプル
    ///     * bool: ブラウザ API が呼ばれたかどうか
    ///     * Option<RuntimeValue>: ブラウザ API の呼び出しによって得られた結果
    fn call_browser_api(
        &mut self,
        func: &RuntimeValue,
        arguments: &[Option<Rc<Node>>],
        env: Rc<RefCell<Environment>>,
    ) -> (bool, Option<RuntimeValue>) {
        // 関数名が document.getElementById の場合、まず1つ目の引数を解釈する。この値は ID 名を表す文字列のはずである。
        if func == &RuntimeValue::StringLiteral("document.getElementById".to_string()) {
            let arg = match self.eval(&arguments[0], env.clone()) {
                Some(a) => a,
                None => return (true, None),
            };
            // ID 名を使用して DOM ツリーから特定の要素を取得する。
            let target = match get_element_by_id(Some(self.dom_root.clone()), &arg.to_string()) {
                Some(n) => n,
                None => return (true, None),
            };
            // DOM ツリーのノードを表す HtmlElement を返す。
            return (
                true,
                Some(RuntimeValue::HtmlElement {
                    object: target,
                    property: None,
                }),
            );
        }
        (false, None)
    }
}

/// RuntimeValue 列挙型
/// JavaScript ランタイムで扱う値を表す。
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    /// https://262.ecma-international.org/#sec-numeric-types
    Number(u64),
    StringLiteral(String),
    HtmlElement {
        object: Rc<RefCell<DomNode>>,
        property: Option<String>,
    },
}

/// Add トレイトの実装
impl Add<RuntimeValue> for RuntimeValue {
    type Output = RuntimeValue;

    fn add(self, rhs: RuntimeValue) -> RuntimeValue {
        if let (RuntimeValue::Number(left_num), RuntimeValue::Number(right_num)) = (&self, &rhs) {
            return RuntimeValue::Number(left_num + right_num);
        }
        RuntimeValue::StringLiteral(self.to_string() + &rhs.to_string())
    }
}

/// Sub トレイトの実装
impl Sub<RuntimeValue> for RuntimeValue {
    type Output = RuntimeValue;

    fn sub(self, rhs: RuntimeValue) -> RuntimeValue {
        if let (RuntimeValue::Number(left_num), RuntimeValue::Number(right_num)) = (&self, &rhs) {
            return RuntimeValue::Number(left_num - right_num);
        }
        // NaN
        RuntimeValue::Number(u64::MIN)
    }
}

/// RuntimeValue の値を文字列に変換する。
impl Display for RuntimeValue {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        let s = match self {
            RuntimeValue::Number(value) => format!("{}", value),
            RuntimeValue::StringLiteral(value) => value.to_string(),
            RuntimeValue::HtmlElement {
                object,
                property: _,
            } => {
                format!("HtmlElement: {:?}", object)
            }
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::js::ast::JsParser;
    use crate::renderer::js::token::JsLexer;

    /// JavaScript に数値のみが含まれる場合のテスト。
    /// 評価結果として数値 (RuntimeValue::Number) が返るはずである。
    #[test]
    fn test_num() {
        let dom = Rc::new(RefCell::new(DomNode::new(DomNodeKind::Document)));
        let input = "42".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new(dom);
        let expected = [Some(RuntimeValue::Number(42))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    /// 足し算のテスト
    #[test]
    fn test_add_nums() {
        let dom = Rc::new(RefCell::new(DomNode::new(DomNodeKind::Document)));
        let input = "1 + 2".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new(dom);
        let expected = [Some(RuntimeValue::Number(3))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    /// 引き算のテスト
    #[test]
    fn test_sub_nums() {
        let dom = Rc::new(RefCell::new(DomNode::new(DomNodeKind::Document)));
        let input = "2 - 1".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new(dom);
        let expected = [Some(RuntimeValue::Number(1))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    /// 変数定義のテスト
    /// var foo=42; を入力するテストを行う。
    /// Program の body に含まれる文を評価すると、None が返るはずである(変数の定義式自体は値を返さない)。
    #[test]
    fn test_assign_variable() {
        let dom = Rc::new(RefCell::new(DomNode::new(DomNodeKind::Document)));
        let input = "var foo=42;".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new(dom);
        let expected = [None];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    /// 変数呼び出しのテスト
    /// var foo=42; foo+1 を入力するテストを行う。
    /// Program の body に含まれる文を評価すると、最初の文は None, 次の文は 43 の値を含む RuntimeValue になるはずである。
    #[test]
    fn test_add_variable_and_num() {
        let dom = Rc::new(RefCell::new(DomNode::new(DomNodeKind::Document)));
        let input = "var foo=42; foo+1".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new(dom);
        let expected = [None, Some(RuntimeValue::Number(43))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    /// 変数変換のテスト
    /// var foo=42; foo=1; foo を入力するテストを行う。
    /// Program の body に含まれる文を評価すると、最初の文は None, 次の文は None, 最後は 1 の値を含む RuntimeValue になるはずである。
    #[test]
    fn test_reassign_variable() {
        let dom = Rc::new(RefCell::new(DomNode::new(DomNodeKind::Document)));
        let input = "var foo=42; foo=1; foo".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new(dom);
        let expected = [None, None, Some(RuntimeValue::Number(1))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    /// 関数定義、呼び出しのテスト
    #[test]
    fn test_add_function_and_num() {
        let dom = Rc::new(RefCell::new(DomNode::new(DomNodeKind::Document)));
        let input = "function foo() { return 42; } foo()+1".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new(dom);
        let expected = [None, Some(RuntimeValue::Number(43))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    /// 引数付き関数定義、呼び出しのテスト
    #[test]
    fn test_define_function_with_args() {
        let dom = Rc::new(RefCell::new(DomNode::new(DomNodeKind::Document)));
        let input = "function foo(a, b) { return a + b; } foo(1, 2) + 3;".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new(dom);
        let expected = [None, Some(RuntimeValue::Number(6))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    /// ローカル変数のテスト
    #[test]
    fn test_local_variable() {
        let dom = Rc::new(RefCell::new(DomNode::new(DomNodeKind::Document)));
        let input = "var a=42; function foo() { var a=1; return a;} foo() + a".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new(dom);
        let expected = [None, None, Some(RuntimeValue::Number(43))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }
}
