use futures::{future::{BoxFuture, FutureExt}, poll, task::{waker_ref, ArcWake}};
use std::{future::Future, sync::mpsc::{sync_channel, Receiver, SyncSender}, sync::{Arc, Mutex}, task::Context, time::Duration, thread};
// The timer we wrote in the previous section:
use async_test::TimerFuture;

/// Task executor that receives tasks off of a channel and runs them.
struct Executor {
    ready_queue: Receiver<Arc<Task>>,
}

/// `Spawner` spawns new futures onto the task channel.
#[derive(Clone)]
struct Spawner {
    task_sender: SyncSender<Arc<Task>>,
}

/// A future that can reschedule itself to be polled by an `Executor`.
struct Task {
    //共享的指针，多个实例化的task可以同时modify，这里约等于TimerFuture
    future: Mutex<Option<BoxFuture<'static, ()>>>,
    //这里的sender是克隆的spawner的，若两处指针都释放则，channel关闭
    task_sender: SyncSender<Arc<Task>>,
}

impl Drop for Task {
    fn drop(&mut self) {
        println!("这里drop的")
    }
}

fn new_executor_and_spawner() -> (Executor, Spawner) {
    const MAX_QUEUED_TASKS: usize = 10_000;
    let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASKS);
    (Executor { ready_queue }, Spawner { task_sender })
}

impl Spawner {
    fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        });
        self.task_sender.send(task).expect("too many tasks queued");
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        println!( "[{:?}]: wake_by_res...", thread::current().id());
        let cloned = arc_self.clone();
        arc_self
            .task_sender
            .send(cloned)
            .expect("too many tasks queued");
    }
}

impl Executor {
    fn run(&self) {
        let mut times = 0;
        println!("[{:?}] start run...",thread::current().id());
        while let Ok(task) = self.ready_queue.recv() {
            println!( "[{:?}]: the {} loop start...", thread::current().id(), times);

            let mut future_slot = task.future.lock().unwrap();
            println!("[{:?}]: 获取task中的future",thread::current().id());
            if let Some(mut future) = future_slot.take() {
                println!("[{:?}] 开始创建waker...",thread::current().id());
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&waker);
                if times==0 {
                    println!("[{:?}] 第{}调用poll，先实例化...",thread::current().id(),times);
                }else {
                    println!("[{:?}] 第{}调用poll，跳过实例化...",thread::current().id(),times);
                }
                // poll方法会去执行future，即async块的第一句
                // 第二次poll时返回ready时，async块执行完成
                if future.as_mut().poll(context).is_pending() {
                    println!("[{:?}] pending后的处理...",thread::current().id());
                    //future_slot是使用take拿走所有权了，现在放回去
                    // 跳过这个if那么future会被释放，就没有sender了
                    *future_slot = Some(future);
                }
            }
            //task_slot没有放回，那么它里面就是None，第三次循环recv失败，run方法结束
            println!( "[{:?}]: the {} loop end...", thread::current().id(), times);
            times += 1;
        }
    }
}

fn main() {
    let (executor, spawner) = new_executor_and_spawner();

    // 先发送一个task，但是future没有调用，等到实际用到async块的时候才执行里面的code
    spawner.spawn(async {
        println!("[{:?}]: howdy!",thread::current().id());
        // 这里会等待futyre完成，包含两次循环，wake后再次poll才能完成
        TimerFuture::new(Duration::new(2, 0)).await;
        println!("[{:?}]: done!",thread::current().id());
    });

    drop(spawner);

    executor.run();

}