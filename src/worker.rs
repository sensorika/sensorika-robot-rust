
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
use serde::de::Deserialize;
use serde::ser::Serialize;
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

type SafeWorker<T> = Arc<Mutex<InnerWorker<T>>>;

enum ChunckOrAck<T>{
    Chunks(Vec<ChunkData<T>>),
    Ack(AckMsg),
}

#[derive(Clone, Debug)]
pub struct ChunkData<T> {
    /// Время когда пришла комманда\отправились данные
    /// Формат: UNIX-время
    pub occur_time: f64,
    pub data: T
}

impl<T: Serialize + Deserialize + Clone + Send> ChunkData<T> {
    fn new(data: T) -> Self{
        ChunkData {
            occur_time: time::now(),
            data: data
        }
    }

    fn to_tuple(&self) -> (f64, T){
        (self.occur_time, self.data.clone())
    }
}


struct InnerWorker<T>{
    name: String,
    dt: f32,
    timer_count: u32,
    data: BufferedQueue<ChunkData<T>>,
    commands: BufferedQueue<ChunkData<T>>,
    is_end: AtomicBool,
    sync_socket: Socket,
    context: Context,
}

impl<T: Serialize + Deserialize + Clone + Send + 'static> InnerWorker<T> {
    fn new(name: &str, ip: &str, port: u32) -> Result<SafeWorker<T>, Box<Error>> {
        let mut ctx = Context::new();
        let mut sync_socket: Socket = try!(ctx.socket(SocketType::REP));
        try!(sync_socket.bind(&format!("tcp://{}:{}", ip, port)));
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
                let mut resp_msg = ChunckOrAck::Ack(AckMsg::wrong());
                if let Ok(v) = w.sync_socket.recv_json(DONTWAIT) {
                    println!("recv");
                    if let Ok(recv_msg) = serde_json::from_value(v){
                        resp_msg = w.route(recv_msg);
                    }else{
                        println!("Not valid JSON");
                        resp_msg = ChunckOrAck::Ack(AckMsg::wrong());
                    }
                    w.send_result_msg(resp_msg);
                }
                w.populate();
                w.timer_count += 10;
                // LOCK OFF
                drop(w);
                thread::sleep_ms(10);
            }
        });
        let iw2 = thread_safe_worker.clone();
        Ok(iw2)
    }

    fn populate(&self){

    }

    fn route(&mut self, msg: Msg) -> ChunckOrAck<T> {
        match msg {
            Msg{action: Actions::call, ..} => unimplemented!(),
            Msg{action: Actions::source, ..} => unimplemented!(),
            Msg{action: Actions::line, ..} => unimplemented!(),
            Msg{action: Actions::get, count: Some(c), ..} => {
                return ChunckOrAck::Chunks(self.data.take(1));
            },
            Msg{action: Actions::get, count: None, ..} => {
                return ChunckOrAck::Chunks(self.data.take(1));
            },
            Msg{action: Actions::set, count: None, data: Some(value) } => {
                if let Ok(new_data) = serde_json::from_value::<T>(value) {
                    self.commands.push(ChunkData::new(new_data));
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

    fn send_result_msg(&mut self, resp_msg: ChunckOrAck<T>){
        match resp_msg {
            ChunckOrAck::Ack(ack) =>{
                let v = serde_json::to_value(&ack);
                self.sync_socket.send_json(&v, DONTWAIT);
            },

            ChunckOrAck::Chunks(chuncks) =>{
                let chks: Vec<(f64, T)> = chuncks
                    .into_iter()
                    .map(|c| c.to_tuple())
                    .collect();
                let v = serde_json::to_value(&chks);
                self.sync_socket.send_json(&v, DONTWAIT);
            },
        }
    }

}

pub struct Worker<T> {
    inner_worker: SafeWorker<T>
}

impl<T: Serialize + Deserialize + Clone + Send + 'static> Worker<T> {
    pub fn new(name: &str, ip: &str, port: u32) -> Result<Worker<T>, Box<Error>>{
        let iw = try!(InnerWorker::new(name, ip, port));
        Ok(Worker{inner_worker: iw})
    }

    /// Добавляет новое значение
    pub fn add(&mut self, v: T){
        let mut w = self.inner_worker.lock().unwrap();
        w.data.push(ChunkData::new(v));
    }

    /// Возвращает N последних присланных сообщений
    pub fn get(&self, n: u32) -> Vec<ChunkData<T>> {
        let mut w = self.inner_worker.lock().unwrap();
        w.data.take(n)
    }

}

impl<T> Drop for Worker<T> {
    fn drop(&mut self){
        let mut w = self.inner_worker.lock().unwrap();
        w.is_end.store(true, Ordering::Relaxed);
    }
}

/*
Этот текст будет удален когда я удостоверюсь на 100%, что
текущая многопоточная реализация воркера нормально работает.

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

Ответ с гиттера:
@Troxid насчёт scoped, суть в следующем: для обычных потоков (thread:: spawn)существует проблема,
что нельзя в него передать не static ссылки, т.к. дочерний поток может пережить основной.
В случае thread::scoped, все дочерние потоки должны будут умереть раньше основного,
из-за этого им можно передавать ссылки из основного. Ну а насчёт синхронизации,
то способов синхронизации уйма, тут уж по нужде/желанию, проще всего использовать каналы,
они собственно для этого и нужны.
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

        }).unwrap();
        h.join().unwrap();
        println!("hello ");
    }
}
