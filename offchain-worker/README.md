

1.在 Offchain Worker 中，使用 Offchain Indexing 特性实现从链上向 Offchain Storage 中写入数据

![image](https://github.com/sevenshi/substrate_study/blob/main/offchain-worker/WX20220922-160320%402x.png)


2.使用 js sdk 从浏览器 frontend 获取到前面写入 Offchain Storage 的数据

![image](https://github.com/sevenshi/substrate_study/blob/main/offchain-worker/WX20220924-211756@2x.png)
![image](https://github.com/sevenshi/substrate_study/blob/main/offchain-worker/WX20220924-211830@2x.png)


3.回答链上随机数（如前面Kitties示例中）与链下随机数的区别

链上的随机数一般采用公开信息，比如使用区块的哈希值/时间戳/难度系数等作为随机数源。
链下随机数一般采用第三方提供的随机数，这种情况通常是中心化的解决方案，通过一个可信的 Oracle 来提供独立的随机数源。



（可选）在 Offchain Worker 中，解决向链上发起不签名请求时剩下的那个错误。
参考：https://github.com/paritytech/substrate/blob/master/frame/examples/offchain-worker/src/lib.rs
（可选）构思一个应用场景，描述如何使用 Offchain Features 三大组件去实现它
（可选）如果有时间，可以实现一个上述原型


实现一个oracle
https://api.coincap.io/v2/assets/polkadot
用Offchain Worker http请求获取最近的10个Polkadot价格储存到Offchain Worker Storage计算平均值
然后用Offchain  Worker 的签名交易发送到链上存储

18位精度的Polkadot price
![image](https://github.com/sevenshi/substrate_study/blob/main/offchain-worker/WX20220924-211856@2x.png)