
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
use std::fmt::Debug;

use zmq::SocketType;
use zmq::Socket;
use zmq::Context;
use zmq::DONTWAIT;
use zmq;

use ini::Ini;
use ini::ini::Properties;

use std::net::TcpStream;
use rand;
use rand::distributions::{IndependentSample, Range};
use util::bindrandomport::*;

type SafeWorker<T> = Arc<Mutex<InnerWorker<T>>>;

enum ChunckOrAck<T>{
    Chunks(Vec<ChunkData<T>>),
    Ack(AckMsg),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

impl<T: Serialize + Deserialize + Clone + Send + 'static + Debug> InnerWorker<T> {
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
            }
        });
        let iw2 = thread_safe_worker.clone();
        Ok(iw2)
    }

    //TODO: implement populate
    fn populate(&self){

    }

    fn route(&mut self, msg: Msg) -> ChunckOrAck<T> {
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

fn is_exist_ini_config() -> bool {
    Ini::load_from_file(".workerrc").is_ok()
}

fn get_random_available_port() -> u16 {
    let mut ctx: Context = zmq::Context::new();
    let mut sock: Socket = ctx.socket(SocketType::ROUTER).unwrap();
    let r = sock.bind_to_random_port("tcp://127.0.0.1");
    r.unwrap() as u16
}

impl<T> Worker<T>
where T: Serialize + Deserialize + Clone + Send + 'static + Debug {

    pub fn new(name: &str, ip: &str, port: u32) -> Result<Worker<T>, Box<Error>>{
        let iw = try!(InnerWorker::new(name, ip, port));
        Ok(Worker{inner_worker: iw})
    }

    /// Создает воркера из локально конфига (.worker_config).
    /// Конфиг должен находится рядом
    /// Если конфиг или секция воркера в конфиге будет отсутствовать,
    /// то будет создан конфиг\секция с случайным доступным портом.
    /// #Arguments
    /// * `worker_name` - имя воркера
    pub fn from_config(worker_name: &str) -> Result<Worker<T>, Box<Error>> {
        let file_name = ".worker_config";

        // if config does not exist
        if !is_exist_ini_config() {
            Ini::new().write_to_file(file_name);
        }
        let mut conf_file = try!(Ini::load_from_file(file_name));
        let mut cf: Ini = conf_file;

        // if config does not contain section with worker name
        if cf.section_mut(Some(worker_name)).is_none(){
            let rand_num: u16 = get_random_available_port();
            cf.with_section(Some(worker_name))
              .set("port", rand_num.to_string());
            cf.write_to_file(file_name);
        }

        let section = try!(cf.section_mut(Some(worker_name)).ok_or("not found name section"));
        let str_port = try!(section.get("port").ok_or("not found port field"));
        let port: u32 = try!(str_port.parse::<u32>());
        Worker::new(worker_name, "127.0.0.1", port)
    }

    /// Добавляет новое значение
    pub fn add(&mut self, v: T){
        let mut w = self.inner_worker.lock().unwrap();
        w.data.push(ChunkData::new(v));
    }

    /// Возвращает N последних присланных команд
    pub fn get(&self, n: u32) -> Vec<ChunkData<T>> {
        let mut w = self.inner_worker.lock().unwrap();
        w.commands.take(n)
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

    use zmq;
    use serde_json;
    use std::thread;
    use zmq::SocketType;
    use zmq::Socket;
    use zmq::Context;
    use zmq::DONTWAIT;
    use message::Msg;
    use util::sendrecvjson::SendRecvJson;
    use worker::Worker;
    use serde_json::Value;
    use serde_json::builder::ObjectBuilder;
    use std::fmt::Debug;
    use ini::Ini;
    use ini::ini::Properties;
    use super::get_random_available_port;

    static IP: &'static str = "127.0.0.1";
    const PORT: u32 = 18000;

    fn create_and_send(msg: &Msg, port: u32) -> Value {
        let mut c = zmq::Context::new();
        let mut s: zmq::Socket = c.socket(SocketType::REQ).unwrap();
        d(line!());
        s.connect(format!("tcp://{}:{}", IP, port).as_str()).unwrap();
        d(line!());
        let v = serde_json::to_value(msg);
        println!("value client REQ: {:?}", &v);
        d(line!());
        s.send_json(&v, 0).unwrap();
        d(line!());
        thread::sleep_ms(200);
        d(line!());
        s.recv_json(0).unwrap()
    }


    #[test]
    fn test_get_from_client(){
        let port = PORT;
        let mut w = Worker::<i32>::new("test", IP, port).unwrap();
        for i in 0..10 {
            w.add(i);
        }

        let v: Value = create_and_send(&Msg::get(3), port);
        let data0 = v.pointer("/0/1").unwrap().as_u64();
        let data1 = v.pointer("/1/1").unwrap().as_u64();
        let data2 = v.pointer("/2/1").unwrap().as_u64();

        assert_eq!(data0, Some(9));
        assert_eq!(data1, Some(8));
        assert_eq!(data2, Some(7));
    }

    #[test]
    fn test_format_msg(){
        let port = PORT + 1;
        let mut w = Worker::<i32>::new("test", IP, port).unwrap();
        for i in 0..10 {
            w.add(i);
        }

        let v: Value = create_and_send(&Msg::get(1), port);
        let data0 = v.pointer("/0/1").unwrap().as_u64();
        let time0 = v.pointer("/0/0").unwrap().as_f64().unwrap();

        // fail, if you can travel back in time ( ͡° ͜ʖ ͡°)
        assert!(time0 > 1474196199f64);
        assert_eq!(data0, Some(9));
    }

    #[test]
    fn test_set_and_get_local(){
        let port = PORT + 2;
        let mut w: Worker<i64> = Worker::<i64>::new("test", IP, port).unwrap();
        d(line!());
        let v: Value = create_and_send(&Msg::set(Value::I64(99)), port);
        d(line!());
        let status = v.pointer("/status").unwrap().as_str();
        d(line!());
        assert_eq!(status, Some("ok"));
        let data0 = w.get(10)[0].data;
        assert_eq!(data0, 99);
    }

    #[test]
    fn test_set_and_get(){
        let port = PORT + 3;
        let mut w: Worker<i64> = Worker::<i64>::new("test", IP, port).unwrap();

        let mut c = zmq::Context::new();
        let mut s: zmq::Socket = c.socket(SocketType::REQ).unwrap();
        s.connect(format!("tcp://{}:{}", IP, port).as_str()).unwrap();

        // Send commands from connector to worker
        for i in 0..10{
            let mut v = serde_json::to_value(&Msg::set(Value::I64(i)));
            println!("value client REQ: {:?}", &v);
            s.send_json(&v, 0).unwrap();
            let v: Value = s.recv_json(0).unwrap();
            let status = v.pointer("/status").unwrap().as_str();
            assert_eq!(status, Some("ok"));
        }

        // Worker compute commands
        let res: Vec<i64> = w.get(10).into_iter().map(|ch| ch.data + 10).collect();
        for el in res.into_iter().rev() {
            w.add(el);
        }

        // Connector wants get computed data
        let mut v = serde_json::to_value(&Msg::get(3));
        s.send_json(&v, 0).unwrap();
        let v: Value = s.recv_json(0).unwrap();
        println!("{:?}", &v);

        let data0 = v.pointer("/0/1").unwrap().as_i64();
        let data1 = v.pointer("/1/1").unwrap().as_i64();
        let data2 = v.pointer("/2/1").unwrap().as_i64();

        assert_eq!(data0, Some(9 + 10));
        assert_eq!(data1, Some(8 + 10));
        assert_eq!(data2, Some(7 + 10));
    }

    #[test]
    fn test_worker_from_existing_conf(){
        let mut conf: Ini = Ini::load_from_file(".worker_config").unwrap();
        let name = "integer_worker";
        conf.with_section(Some(name))
            .set("port", "7777");

        let w = Worker::<i32>::from_config(name);
        assert!(w.is_ok());
    }

    #[test]
    fn test_worker_from_empty_conf(){
        let mut conf: Ini = Ini::load_from_file(".worker_config").unwrap();
        conf.clear();

        let w = Worker::<i32>::from_config(name);
        assert!(w.is_ok());
    }

    #[test]
    fn test_get_random_port(){
        let fst = get_random_available_port();
        let snd = get_random_available_port();
        println!("{}, {}", fst, snd);
        assert!(fst != snd);
    }

    fn d<T: Debug>(any: T){
        println!("d: {:?}", any);
    }
}
