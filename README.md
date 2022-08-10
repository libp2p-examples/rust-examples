# rust-examples
# 这是一个 libp2p 的rust 使用例子，原始代码几乎来自
rust-libp2p/examples
教程网址 http://www.libp2p.net.cn
```
git clone https://github.com/libp2p-examples/rust-examples
cd rust-examples
cargo build --example chat
```

# 编译第一个例子

```
cargo build --example 01-peer
```

# 增加了自制协议 放在 src目录下的三个文件
```
src/
├── handler.rs
├── lib.rs
└── protocol.rs

```
本协议 仅仅支持 收发字符串，使用用例见 examples/03-msg.rs

```
cargo build --example 03-msg
```

开启两个 用例发送消息
```
./target/debug/examples/03-msg
Local peer id: PeerId("12D3KooWSp7toxzXPHZvpf3STUWVZaKxFYBKDj2An1UJj5CgddUC")
Listening on "/ip4/127.0.0.1/tcp/36387"
Listening on "/ip4/192.168.1.17/tcp/36387"

```

```
./target/debug/examples/03-msg /ip4/192.168.1.17/tcp/36387
```

这样就可以敲键盘互相发送消息了。
