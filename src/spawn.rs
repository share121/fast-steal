extern crate alloc;
use crate::{split_task::SplitTask, task::Task, task_list::TaskList};
use alloc::sync::Arc;
use std::thread::{self, JoinHandle};

pub trait Spawn {
    fn spawn<F>(self, threads: usize, action: F) -> JoinHandle<()>
    where
        F: FnOnce(Arc<Task>, &dyn Fn() -> bool) + Send + Clone + 'static;
}

impl Spawn for Arc<TaskList> {
    fn spawn<F>(self, threads: usize, action: F) -> JoinHandle<()>
    where
        F: FnOnce(Arc<Task>, &dyn Fn() -> bool) + Send + Clone + 'static,
    {
        thread::spawn(move || {
            thread::scope(|s| {
                let tasks: Arc<Vec<Arc<Task>>> = Arc::new(
                    Task::from(self.as_ref())
                        .split_task(threads)
                        .map(|t| Arc::new(t))
                        .collect(),
                );
                for id in 0..threads {
                    let tasks = tasks.clone();
                    let action = action.clone();
                    s.spawn(move || {
                        action(tasks[id].clone(), &|| {
                            let (max_pos, max_remain) = tasks
                                .iter()
                                .enumerate()
                                .map(|(i, w)| (i, w.remain()))
                                .max_by_key(|(_, remain)| *remain)
                                .unwrap();
                            if max_remain < 2 {
                                return false;
                            }
                            let (start, end) = tasks[max_pos].split_two();
                            tasks[id].set_start(start);
                            tasks[id].set_end(end);
                            true
                        })
                    });
                }
            });
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        collections::{HashMap, hash_map::Entry},
        sync::{Arc, mpsc},
    };

    fn fib(n: usize) -> usize {
        match n {
            0 => 0,
            1 => 1,
            _ => fib(n - 1) + fib(n - 2),
        }
    }

    fn fib_fast(n: usize) -> usize {
        let mut a = 0;
        let mut b = 1;
        for _ in 0..n {
            (a, b) = (b, a + b);
        }
        a
    }

    #[test]
    fn test_spawn() {
        let tasks: Arc<TaskList> = Arc::new(vec![0..48].into());
        let (tx, rx) = mpsc::channel();
        let tasks_clone = tasks.clone();
        let handle = tasks.clone().spawn(8, move |task, get_task| {
            loop {
                while task.start() < task.end() {
                    let i = tasks_clone.get(task.start());
                    tx.send((i, fib(i))).unwrap();
                    task.fetch_start(1);
                }
                if !get_task() {
                    break;
                }
            }
        });
        // 汇总任务结果
        let mut data = HashMap::new();
        for (i, res) in rx {
            // 如果重复计算就报错
            match data.entry(i) {
                Entry::Occupied(_) => panic!("数字 {i}，值为 {res} 重复计算"),
                Entry::Vacant(entry) => {
                    entry.insert(res);
                }
            }
            data.insert(i, res);
        }
        // 等待任务结束
        handle.join().unwrap();
        // 验证结果
        dbg!(&data);
        for i in 0..tasks.len {
            let index = tasks.get(i);
            assert_eq!((index, data.get(&index)), (index, Some(&fib_fast(index))));
        }
    }
}
