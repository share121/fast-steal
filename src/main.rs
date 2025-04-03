use fast_steal::{spawn::spawn, split_task::SplitTask, task::Task};
use std::{
    collections::{HashMap, hash_map::Entry},
    sync::mpsc,
};

fn fib(n: u128) -> u128 {
    match n {
        0 => 0,
        1 => 1,
        _ => fib(n - 1) + fib(n - 2),
    }
}

fn main() {
    // 设定任务
    let tasks = vec![Task {
        start: 0u128,
        end: 44u128,
    }];
    // 切分任务
    let task_group = tasks.split_task(8);
    // 接受任务结果
    let (tx, rx) = mpsc::channel();
    let handle = spawn(task_group, move |rx_task, progress| {
        let mut tasks_gobal = Some(rx_task.recv().unwrap());
        'task: loop {
            match tasks_gobal.take() {
                Some(ref tasks) => {
                    // 退出条件
                    if tasks.is_empty() {
                        return;
                    }
                    // 业务逻辑
                    for task in tasks {
                        for i in task.start..task.end {
                            // 任务窃取过程，必须放在任务前
                            match rx_task.try_recv() {
                                Ok(task) => {
                                    tasks_gobal = Some(task);
                                    continue 'task;
                                }
                                Err(e) => match e {
                                    mpsc::TryRecvError::Empty => {}
                                    mpsc::TryRecvError::Disconnected => {
                                        panic!("{:?}", e);
                                    }
                                },
                            }
                            // 返回任务执行进度，必须放在任务前
                            progress(1);
                            // println!("开始计算 {}", i);
                            // 任务执行
                            let res = fib(i);
                            // 任务结果发送
                            tx.send((i, res)).unwrap();
                        }
                    }
                }
                None => {
                    tasks_gobal = Some(rx_task.recv().unwrap());
                }
            }
        }
    });
    // 汇总任务结果
    let mut data = HashMap::new();
    for (i, res) in rx {
        // 如何重复计算就报错
        match data.entry(i) {
            Entry::Occupied(_) => panic!("数字 {i}，值为 {res} 重复计算"),
            Entry::Vacant(entry) => {
                entry.insert(res);
            }
        }
    }
    // 等待任务结束
    handle.join().unwrap();
    // 验证结果
    dbg!(&data);
    for i in tasks[0].start..tasks.last().unwrap().end {
        assert_eq!((i, data.get(&i)), (i, Some(&fib(i))));
    }
}
