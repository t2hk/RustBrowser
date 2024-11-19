use alloc::string::String;

/// エラー列挙型
/// エラーの種類を表す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
  Network(String),
  UnexpectedInput(String),
  InvalidUI(String),
  Other(String),
}