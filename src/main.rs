use fast_steal::{spawn::Spawn, split_task::SplitTask, task::Task};
use std::collections::{HashMap, hash_map::Entry};

fn fib(n: u128) -> u128 {
    match n {
        0 => 0,
        1 => 1,
        _ => fib(n - 1) + fib(n - 2),
    }
}

fn fib_fast(n: u128) -> u128 {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        (a, b) = (b, a + b);
    }
    a
}

fn fun() {
    // 设定任务
    let tasks = vec![Task {
        start: 0u128,
        end: 44u128,
    }];
    // 切分任务
    let task_group = tasks.split_task(8);
    // 接受任务结果
    let (tx, rx) = crossbeam_channel::unbounded();
    let handle = task_group.spawn(move |rx_task, id, progress| {
        println!("线程 {id} 启动");
        // 监听任务
        'task: for tasks in &rx_task {
            // 退出条件
            if tasks.is_empty() {
                break;
            }
            // 业务逻辑
            for task in tasks {
                for i in task.start..task.end {
                    // 任务窃取过程，必须放在任务前
                    if !rx_task.is_empty() {
                        continue 'task;
                    }
                    // 返回任务执行进度，必须放在任务前
                    progress(1);
                    // 任务执行
                    let res = fib(i);
                    // 任务结果发送
                    tx.send((i, res)).unwrap();
                }
            }
        }
    });
    // 汇总任务结果
    let mut data = HashMap::new();
    for (i, res) in rx {
        // 如果重复计算就报错
        match data.entry(i) {
            Entry::Occupied(_) => println!("数字 {i}，值为 {res} 重复计算"),
            Entry::Vacant(entry) => {
                entry.insert(res);
            }
        }
        data.insert(i, res);
    }
    // 等待任务结束
    handle.join().unwrap();
    // 验证结果
    // dbg!(&data);
    for i in tasks[0].start..tasks.last().unwrap().end {
        assert_eq!((i, data.get(&i)), (i, Some(&fib_fast(i))));
    }
}

fn main() {
    for i in 0..100 {
        println!("{i}");
        fun();
    }
}
