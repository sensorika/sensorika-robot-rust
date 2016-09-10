
use std::str;
use std::error::Error;
use std::result::*;
use std::time::SystemTime;
use util::time;
use util::buffered_queue::BufferedQueue;
use util::sendrecvjson::SendRecvJson;
use util::sendrecvjson::SendRecvMode;
use serde_json::{Value, Map};
use serde_json;
use std::str::FromStr;
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::sync::Arc;
use std::sync::Mutex;
use super::message::{Msg, Actions, AckMsg};

use zmq::SocketType;
use zmq::Socket;
use zmq::Context;
use zmq::DONTWAIT;

type SafeWorker = Arc<Mutex<InnerWorker>>;

enum ChunckOrAck{
    Chunks(Vec<ChunkData>),
    Ack(AckMsg),
}

#[derive(Clone)]
pub struct ChunkData {
    /// Время когда пришла комманда\отправились данные
    /// Формат: UNIX-время
    pub occur_time: f64,
    /// Строка содержащая валидный JSON
    pub json_data: String
}

impl ChunkData {
    fn new(data: String) -> Self{
        ChunkData {
            occur_time: time::now(),
            json_data: data
        }
    }

    fn to_tuple(&self) -> (f64, String){
        (self.occur_time, self.json_data.clone())
    }
}

impl Default for ChunkData {
    fn default() -> Self {
        ChunkData::new(r#"{"empty":null}"#.into())
    }
}

struct InnerWorker{
    name: String,
    dt: f32,
    timer_count: u32,
    data: BufferedQueue<ChunkData>,
    commands: BufferedQueue<ChunkData>,
    is_end: AtomicBool,
    sync_socket: Socket,
    context: Context,
}

impl InnerWorker {
    fn new(name: &str, ip: &str, port: u32) -> Result<SafeWorker, Box<Error>> {
        let mut ctx = Context::new();
        let mut sync_socket: Socket = try!(ctx.socket(SocketType::REP));
        let mut inner_worker = InnerWorker{
            name: name.into(),
            dt: 0.001,
            timer_count: 0,
            data: BufferedQueue::new(100),
            commands: BufferedQueue::new(100),
            is_end: AtomicBool::new(false),
            sync_socket: sync_socket,
            context: ctx,
        };
        let thread_safe_worker = Arc::new(Mutex::new(inner_worker));
        let iw1 = thread_safe_worker.clone();
        let h = thread::spawn(move ||{
            loop{
                // LOCK ON
                let mut w = iw1.lock().unwrap();

                // CHECK OVERFLOW TIMER COUNTER
                if w.timer_count > 2000 {
                    w.timer_count = 0;
                }
                // CHECK NEEDED END
                if w.is_end.load(Ordering::Relaxed) {
                    break;
                }
                // RECV, ROUTE AND SEND MSG
                let mut resp_msg = ChunckOrAck::Chunks(Vec::default());
                if let Ok(v) = w.sync_socket.recv_json(DONTWAIT) {
                    if let Ok(recv_msg) = serde_json::from_value(v){
                        resp_msg = w.route(recv_msg);
                    }else{
                        println!("Not valid JSON");
                        resp_msg = ChunckOrAck::Ack(AckMsg::wrong());
                    }
                    w.send_result_msg(resp_msg);
                }
                w.populate();
                w.timer_count += 100;
                // LOCK OFF
                drop(w);
                thread::sleep_ms(100);
            }
        });
        let iw2 = thread_safe_worker.clone();
        Ok(iw2)
    }

    fn populate(&self){

    }

    fn route(&mut self, msg: Msg) -> ChunckOrAck {
        match msg {
            Msg{action: Actions::call, ..} => unimplemented!(),
            Msg{action: Actions::source, ..} => unimplemented!(),
            Msg{action: Actions::line, ..} => unimplemented!(),
            Msg{action: Actions::get, count: Some(c), ..} => {
                return ChunckOrAck::Chunks(self.data.take(c));
            },
            Msg{action: Actions::get, count: None, ..} => {
                return ChunckOrAck::Chunks(self.data.take(1));
            },
            Msg{action: Actions::set, count: None, data: Some(value) } => {
                if let Ok(str) = serde_json::from_value(value) {
                    self.commands.push(ChunkData::new(str));
                    return ChunckOrAck::Ack(AckMsg::ok());
                }else{
                    return ChunckOrAck::Ack(AckMsg::wrong());
                }
            },
            _ => {
                println!("Not valid format of JSON");
                return ChunckOrAck::Ack(AckMsg::wrong());
            }
        }
    }

    fn send_result_msg(&mut self, resp_msg: ChunckOrAck){
        match resp_msg {
            ChunckOrAck::Ack(ack) =>{
                let v = serde_json::to_value(&ack);
                self.sync_socket.send_json(&v, DONTWAIT);
            },

            ChunckOrAck::Chunks(chuncks) =>{
                let chks: Vec<(f64, String)> = chuncks
                    .into_iter()
                    .map(|c| c.to_tuple())
                    .collect();
                let v = serde_json::to_value(&chks);
                self.sync_socket.send_json(&v, DONTWAIT);
            },
        }
    }
}

pub struct Worker {
    inner_worker: SafeWorker
}

impl Worker {
    pub fn new(name: &str, ip: &str, port: u32) -> Result<Worker, Box<Error>>{
        let iw = try!(InnerWorker::new(name, ip, port));
        Ok(Worker{inner_worker: iw})
    }

    /// Добавляет новое значение
    pub fn add(&mut self, v: &Value){
        let mut w = self.inner_worker.lock().unwrap();
        if let Ok(str) = serde_json::to_string(&v){
            w.data.push(ChunkData::new(str));
        }
    }

    /// Возвращает N последних присланных сообщений
    pub fn get(&self, n: u32) -> Vec<ChunkData> {
        let mut w = self.inner_worker.lock().unwrap();
//        let mut result: Vec<Value> = Vec::new();
//        let json_strings: Vec<String> = w.data.take(n);
//        for str in json_strings {
//            if let Ok(v) = Value::from_str(str.as_ref()){
//                result.push(v);
//            }
//        }
//        result
        w.data.take(n)
    }

}

impl Drop for Worker {
    fn drop(&mut self){
        let mut w = self.inner_worker.lock().unwrap();
        w.is_end.store(true, Ordering::Relaxed);
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
Заставить клиента использовать низкоурв. апи + блокировки. или сделать обертку
4) Всю структуру отдавать в поток, оставляя снаружи только основных метода
add и get.
add будет передавать значеия в поток цикла через канал.
у get будет блокирующий канал. поток цикла будет
ждать пока прочитают последния значения, а затем загружать в канал новые.

*/

#[cfg(test)]
mod tests{

    use std::thread;
    use zmq::SocketType;
    use zmq::Socket;
    use zmq::Context;
    use zmq::DONTWAIT;
    use super::Worker;

    #[test]
    fn test_worker(){
        let h = thread::Builder::new().name("test worker".into()).spawn(move||{
            let w = Worker::new();

        }).unwrap();
        h.join().unwrap();
    }
}
