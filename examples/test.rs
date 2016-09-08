
use std::time::SystemTime;
use std::time::Duration;
use std::thread;


fn main(){
    let t = SystemTime::now();
    loop{
        thread::sleep_ms(500);
        let res = t.elapsed();
        println!("{:?}",&res);
    }
}
