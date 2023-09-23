use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context,Waker, Poll},
    thread,
    time::Duration
};
#[derive(Debug)]
pub struct TimerFuture{
    shared_state: Arc<Mutex<SharedState>> // 共享的，可以一起修改
}
#[derive(Debug)]
struct SharedState{
    completed: bool,
    waker: Option<Waker>
}

impl Future for TimerFuture{
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!( "[{:?}]: Polling TimeFuture",thread::current().id());
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            println!( "[{:?}]: TimeFuture completed...",thread::current().id());
            Poll::Ready(())
        }else {
            shared_state.waker = Some(cx.waker().clone());
            println!( "[{:?}]: TimeFuture Pending...",thread::current().id());
            Poll::Pending
        }
    }
}

impl TimerFuture {
    pub fn new(duration:Duration) -> Self{
        let shared_state = Arc::new(Mutex::new(SharedState{
            completed: false,
            waker: None
        }));

        let thread_shared_state = shared_state.clone();
        thread::spawn(
            move || {
                // 开启一个线程，这里代表一段代码，执行一段时间
                for i in 0..duration.as_secs() {
                    println!( "[{:?}]: 睡眠第{}s...", thread::current().id(),i);
                }
                thread::sleep(duration);
                let mut shared_state = thread_shared_state.lock().unwrap();
                shared_state.completed = true;
                if let Some(waker) = shared_state.waker.take(){
                    println!( "[{:?}]: TimerFuture获得waker，进行wake()...", thread::current().id());
                    waker.wake();
                }
                else {
                    println!( "[{:?}]: TimerFuture没有获得waker", thread::current().id())
                }
            }
        );
        // 用None填补没有获取到的值,然后开启一个线程获取资源后，然后告诉程序TimerFuture更新，可以调用wake方法了...",
        println!("[{:?}] 完成创建，返回新的TimerFuture, 其中SharedState的waker为None",thread::current().id());
        TimerFuture{shared_state}
    }
}
