use std::collections::LinkedList;

pub struct BufferedQueue<T>{
    list: LinkedList<T>,
    size: usize
}

impl<T: Clone> BufferedQueue<T>{

    /// Создает новую буферезированную очередь
    /// с размерностью `size`
    /// Сначала размер буффера равен 0
    /// В процессе заполнения, буффер расширяется до `size`
    /// #Arguments
    /// * `size` - размер буфера
    ///
    /// #Panics
    /// Когда `size` == 0
    ///
    /// #Example
    /// ```ignore
    /// extern crate sensorika;
    /// use sensorika::util::buffered_queue::BufferedQueue;
    ///
    /// let mut buffer: BufferedQueue<i32> = BufferedQueue::new(10);
    /// ```
    pub fn new(size: usize) -> BufferedQueue<T>{
        assert!(size != 0);
        BufferedQueue{
            list: LinkedList::new(),
            size: size
        }
    }

    /// Добавляет в очередь один элемент.
    /// Если очередь переполнена, то
    /// элементы, которые находятся
    /// в конце очереди (слева) - уничтожаются.
    ///
    /// # Arguments
    /// * `el` - добавляемый элемент
    ///
    /// # Examples
    /// ```ignore
    /// extern crate sensorika;
    /// use sensorika::util::buffered_queue::BufferedQueue;
    ///
    /// let mut buffer: BufferedQueue<i32> = BufferedQueue::new(2);
    /// assert_eq!(buffer.take_all(), vec![0,  0]);
    /// buffer.push(10);
    /// assert_eq!(buffer.take_all(), vec![10, 0]);
    /// buffer.push(10);
    /// assert_eq!(buffer.take_all(), vec![10, 10]);
    /// buffer.push(20);
    /// assert_eq!(buffer.take_all(), vec![20, 10]);
    /// ```
    ///
    pub fn push(&mut self, el: T){
        if self.list.len() < self.size {
            self.list.push_front(el);
        }else{
            self.list.push_front(el);
            let _ = self.list.pop_back();
        }
    }

    /// Возвращает N первых элементов из очереди (справа).
    /// Где N - `size`
    ///
    /// #Arguments
    /// * `num` - количество забираемых элементов
    ///
    /// #Examples
    /// ```ignore
    /// extern crate sensorika;
    /// use sensorika::util::buffered_queue::BufferedQueue;
    ///
    /// let n = 4;
    /// let mut buffer: BufferedQueue<i32> = BufferedQueue::new(10);
    /// buffer.push(1);
    /// buffer.push(2);
    /// buffer.push(3);
    /// let xs = buffer.take(n);
    /// assert_eq!(xs, vec![1, 2, 3, 0]);
    /// assert_eq!(xs.len(), n);
    /// ```
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

    /// Возвращает все элементы из очереди.
    ///
    /// #Examples
    /// ```ignore
    /// extern crate sensorika;
    /// use sensorika::util::buffered_queue::BufferedQueue;
    ///
    /// let size = 10;
    /// let mut buffer: BufferedQueue<i32> = BufferedQueue::new(size);
    /// assert_eq!(buffer.take_all().len(), size);
    /// ```
    pub fn take_all(&self) -> Vec<T> {
        let mut result: Vec<T> = Vec::new();
        for el in self.list.iter() {
            result.push(el.clone());
        }
        result
    }

    pub fn size(&self) -> usize {
        self.list.len()
    }
}

#[cfg(test)]
mod tests{
    use super::BufferedQueue;

    #[test]
    fn test_push_and_take(){
        let mut b = BufferedQueue::<i32>::new(4);
        b.push(0);
        b.push(10);
        b.push(20);
        b.push(30);
        b.push(40);
        let first_four_number = b.take(4);
        assert_eq!(first_four_number, vec![40, 30, 20, 10]);
    }

    #[test]
    fn test_small_buffer(){
        let mut b = BufferedQueue::<i32>::new(1);
        for i in 0..100_000{
            b.push(i);
        }
        let first = b.take(1);
        assert_eq!(first, vec![99_999]);
    }

    #[test]
    fn test_take_size(){
        let mut b = BufferedQueue::<usize>::new(100);
        for i in 1..100{
            assert_eq!(b.take(i).len(), 0);
        }
    }

    #[test]
    fn test_big_buffer(){
        let mut b = BufferedQueue::<usize>::new(1000);
        let big_n = 10_000_000;
        for i in 0..big_n {
            b.push(i);
        }
        assert_eq!(b.take(2), vec![big_n - 1, big_n -2]);
    }

    #[test]
    fn test_take_all(){
        let size = 10;
        let mut buffer = BufferedQueue::<usize>::new(size);
        for i in 0..size+1{
            buffer.push(i);
        }
        assert_eq!(buffer.take_all().len(), size);
    }

    #[test]
    fn test_list_size(){
        let size = 10;
        let mut buffer = BufferedQueue::<usize>::new(size);
        for i in 0..size+1{
            buffer.push(i);
        }
        assert_eq!(buffer.size(), size as usize);
    }

    #[test]
    fn test_all(){
        let mut buffer = BufferedQueue::<usize>::new(10);
        buffer.push(1);
        buffer.push(2);
        buffer.push(3);
        let xs = buffer.take(3);
        assert_eq!(xs, vec![3, 2, 1]);
        assert_eq!(xs.len() as u32, 3);
    }

    #[test]
    fn test_growing(){
        let n = 4;
        let mut buffer = BufferedQueue::<u32>::new(n);
        for i in 0..10 {
            if i < n {
                assert_eq!(buffer.size(), i);
            }else{
                assert_eq!(buffer.size(), n);
            }
            buffer.push(i as u32);
        }
    }
}

