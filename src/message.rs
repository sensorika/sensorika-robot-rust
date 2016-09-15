#![allow(non_camel_case_types)]

use serde_json;
use serde_json::Value;

///////////////////////////////////////////////////////////////////////
// Command message - собщение-команда для воркера
///////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Debug)]
pub enum Actions{
    call,
    source,
    line,
    get,
    set
}

/// Сообщения которые присылаются от клиента
/// * `action` - действие, необходимое совершить воркеру
/// * `count` - количество данных, с которыми необходимо произвести `action`
/// * `data` - данные, с которыми необходимо произвести `action`
#[derive(Serialize, Deserialize, Debug)]
pub struct Msg {
    pub action: Actions,
    pub count: Option<u32>,
    pub data: Option<Value>,
}

impl Msg {
    pub fn get(n: u32) -> Self {
        Msg{action: Actions::get, count: Some(n), data: None}
    }

    pub fn set(n: u32, data: Value) -> Self {
        Msg{action: Actions::set, count: Some(n), data: Some(data)}
    }
}

///////////////////////////////////////////////////////////////////////
// Acknowledge message - ответное сообщение для коннектора
///////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Debug)]
pub enum Status{
    ok,
    wrong_params,
}

/// Ответ, который отправляет воркер клиенту
/// * `ok` - комманда получена, действие произведено
/// * `wrong_param` - ошибка, не правильные параметры
#[derive(Serialize, Deserialize, Debug)]
pub struct AckMsg{
    pub status: Status
}

impl AckMsg{
    pub fn ok() -> Self {
        AckMsg{status: Status::ok}
    }

    pub fn wrong() -> Self {
        AckMsg{status: Status::wrong_params}
    }
}