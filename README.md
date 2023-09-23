# 执行流程

## 1.主线程执行

1. 创建发送端和接受端
2. 发送端调用spawn方法，将async块封装成Task发送到channel中
3. 接受端while循环revc方法，每次会循环会阻塞，直到接受到Task
4. 取出task中的future，Poll一次
5. 第一次poll后，async块开始执行



![image-20230923180704360](/Users/zackjchen/Library/Application Support/typora-user-images/image-20230923180704360.png)



## 2.async块执行

1. async块执行如图，new 一个TimerFuture
2. new方法返回之前开启了一个新的线程
3. new方法返回TimerFuture如图第二个黄框，然后poll一下
4. 发现poll返回Pending，就会放回future到Task(task放)。
5. 第二次循环开始，revc阻塞

![image-20230923181103380](/Users/zackjchen/Library/Application Support/typora-user-images/image-20230923181103380.png)

## 3.async块开启新线程

1. Sleep(2)代表代码执行需要cost一些时间
2. 完成后TimerFuture中的一些field update完成
   - 这里能更新是因为`Arc<Mutex>`，共享内存，一起修改
3. 新线程结束时调用wake方法



## 4.新线程唤起主线程

下面的都在第三个图

1. Task的wake方法发送一个新的Task到channel中
2. 主线程的Receiver接受到Task，开始第二轮执行

## async块执行结束

此时在主线程执行

1. 第二轮poll发现新线程更新了completed为true，所以结束返回Ready
2. 此时async块执行结束

## 5.主线程执行结束

1. run方法发现ready了，那么task就不会放回，channel关闭，run方法结束

<img src="/Users/zackjchen/Library/Application Support/typora-user-images/image-20230923184050586.png" alt="image-20230923184050586" style="zoom:40%;" />