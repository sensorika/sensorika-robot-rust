use std::collections::LinkedList;
//FIXME: write doc

struct BufferedQueue<T>{
    list: LinkedList<T>,
    size: usize
}

impl<T: Default + Clone> BufferedQueue<T>{

    pub fn new(size: usize) -> BufferedQueue<T>{
        assert!(size != 0);
        let mut tmp_list: LinkedList<T> = LinkedList::new();
        for i in 0..size-1 {
            tmp_list.push_front(T::default());
        }
        BufferedQueue{
            list: tmp_list,
            size: size
        }
    }

    pub fn push(&mut self, el: T){
        if self.list.len() < self.size {
            self.list.push_front(el);
        }else{
            self.list.push_front(el);
            let _ = self.list.pop_back();
        }
    }

    pub fn take(&self, num: u32) -> Vec<T> {
        let mut result: Vec<T> = Vec::new();
        let mut count = 0;
        for el in self.list.iter() {
            if count < num {
                result.push(el.clone());
                count += 1;
            }else {
                break;
            }
        }
        result
    }
}

#[test]
fn test_push_and_take(){
    let mut b: BufferedQueue<i32> = BufferedQueue::new(4);
    // b = [0, 0, 0, 0]
    b.push(10);
    // b = [10, 0, 0, 0]
    b.push(10);
    // b = [10, 10, 0, 0]
    let first_four_number = b.take(4);
    assert_eq!(first_four_number, vec![10, 10, 0, 0]);
}

#[test]
fn test_small_buffer(){
    let mut b: BufferedQueue<i32> = BufferedQueue::new(1);
    for i in 0..100_000{
        b.push(i);
    }
    let first = b.take(1);
    assert_eq!(first, vec![99_999]);
}

#[test]
fn test_take_size(){
    let mut b: BufferedQueue<i32> = BufferedQueue::new(100);
    for i in 0..100{
        assert_eq!(b.take(i).len(), i as usize);
    }
}

#[test]
fn test_big_buffer(){
    let mut b: BufferedQueue<i32> = BufferedQueue::new(1000);
    let big_n = 10_000_000;
    for i in 0..big_n {
        b.push(i);
    }
    assert_eq!(b.take(2), vec![big_n - 1, big_n -2]);
}

#[test]
fn test_always_same_size(){
    for size in 1..1000{
        let mut b: BufferedQueue<i32> = BufferedQueue::new(size);
        for i in 0..1_000 {
            b.push(i);
            assert_eq!(b.list.len(), size);
        }
    }
}