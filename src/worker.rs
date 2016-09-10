
use std::str;
use std::error::Error;
use std::result::*;
use std::time::SystemTime;
use util::buffered_queue::BufferedQueue;
use util::sendrecvjson::SendRecvJson;
use util::sendrecvjson::SendRecvMode;
use serde_json::{Value, Map};
use serde_json;
use std::str::FromStr;
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

use zmq::SocketType;
use zmq::Socket;
use zmq::Context;
use zmq::DONTWAIT;

pub struct Worker {
    dt: f32,
    timer_count: u32,
    data: BufferedQueue<String>,
    /// Флаг для остановки потока `worker`
    is_end: AtomicBool,
    sync_socket: Socket,
    context: Context,
}

impl Worker {
    pub fn new(ip: &str, port: u32) -> Result<Worker, Box<Error>> {
        let mut ctx = Context::new();
        let mut sync_socket: Socket = try!(ctx.socket(SocketType::REP));

        Ok(Worker{
            dt: 0.001,
            timer_count: 0,
            data: BufferedQueue::new(100),
            is_end: AtomicBool::new(false),
            sync_socket: sync_socket,
            context: ctx,
        })
    }

    /// Добавляет новое значение
    pub fn add(&mut self, v: &Value){
        if let Ok(str) = serde_json::to_string(&v){
            self.data.push(str);
        }
    }

    /// Возвращает N последних присланных сообщений
    pub fn get(&self, n: u32) -> Vec<Value> {
        let mut result: Vec<Value> = Vec::new();
        let json_strings: Vec<String> = self.data.take(n);
        for str in json_strings {
            if let Ok(v) = Value::from_str(str.as_ref()){
                result.push(v);
            }
        }
        result
    }

    pub fn populate(&self){
        //FIXME: unimplemented!();
    }

    pub fn run(&mut self){
        thread::spawn(||{self.event_loop()});
    }

    fn event_loop(&mut self){
        loop{
            thread::sleep_ms(100);
            self.timer_count += 100;
            if self.timer_count > 2000 {
                self.timer_count = 0;
            }
            if self.is_end.load(Ordering::Relaxed) {
                break;
            }
            if let Ok(v) = self.sync_socket.recv_json(DONTWAIT) {
                let json = serde_json::to_string(&v).unwrap();
                self.data.push(json);
            }
            self.populate();
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self){
        self.is_end.store(true, Ordering::Relaxed);
    }
}

/*
Есть некоторые проблемы с многопоточностью
состояния воркера шарятся между двумя/тремя тредами:
мэин тред, тред под цикл обработчика + опц под отдельный таймер
Наивная реализация запуска треда в fn run приводит к тому, что
идет "захват" воркера в поток и воркер остается без защитных механизмов
(может вызываться из основого потока)
Способы решения:
1) использовать тред локальной области видимости из crossbeam
2) Все поля покрыть Arc<Mutex<_>>. Есть вероятность схватывать блокировки
3) Всю структуру покрыть Arc<Mutex<_>>.
Заставить клиента использовать низкоурв. апи + блокировки
4) Всю структуру отдавать в поток, оставляя снаружи только основных метода
add и get.
add будет передавать значеия в поток цикла через канал.
у get будет блокирующий канал. поток цикла будет
ждать пока прочитают последния значения, а затем загружать в канал новые.

Наиболее оптимальным решением видится 4ое.
*/

#[cfg(test)]
mod tests{

    use zmq::SocketType;
    use zmq::Socket;
    use zmq::Context;
    use zmq::DONTWAIT;

    #[test]
    fn test_worker(){

    }
}
