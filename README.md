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



# 利用relay 穿透打洞
```
cargo build --example 05-relay
cargo build --example 06-hole_punching
```

# 启动relay server
```
#.\target\debug\examples\05_relay
opt: Opt { use_ipv6: None, secret_key_seed: 0, port: 45678 }
Local peer id: PeerId("12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN")
Listening on "/ip4/192.168.1.105/tcp/45678"
Listening on "/ip4/127.0.0.1/tcp/45678"
```

# 启动客户端 B
```
klang@klang-LAPKC71E:~/rust-examples$ ./target/debug/examples/06-hole_punching --relay-address /ip4/192.168.1.105/tcp/45678
Local peer id: PeerId("12D3KooWCNL5YR6qBW5Xfa3R8tye4ASJACEh9GNGZ81m7T6fsbxx")
Listening on "/ip4/127.0.0.1/tcp/42677"
Listening on "/ip4/192.168.1.108/tcp/42677"
Relay told us our public address: "/ip4/192.168.1.108/tcp/42677"
Registered for namespace 'rendezvous' at rendezvous point 12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN for the next 7200 seconds
Discovered peer 12D3KooWCNL5YR6qBW5Xfa3R8tye4ASJACEh9GNGZ81m7T6fsbxx at /ip4/192.168.1.108/tcp/42677
```

# 启动客户端 A （docker里）
```
root@698d6dd93a3c:/home/user/rust-examples# ./target/debug/examples/06-hole_punching --relay-address /ip4/192.168.1.105/tcp/45678
Local peer id: PeerId("12D3KooWHrxvWRcsQxY9WmMxWwcKMAtsJZw3gDj6aDDqisnbihK8")
Listening on "/ip4/127.0.0.1/tcp/38745"
Listening on "/ip4/172.17.0.2/tcp/38745"
Relay told us our public address: "/ip4/192.168.1.106/tcp/38745"
Registered for namespace 'rendezvous' at rendezvous point 12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN for the next 7200 seconds
Discovered peer 12D3KooWHrxvWRcsQxY9WmMxWwcKMAtsJZw3gDj6aDDqisnbihK8 at /ip4/192.168.1.106/tcp/38745
Discovered peer 12D3KooWCNL5YR6qBW5Xfa3R8tye4ASJACEh9GNGZ81m7T6fsbxx at /ip4/192.168.1.108/tcp/42677
Established connection to PeerId("12D3KooWCNL5YR6qBW5Xfa3R8tye4ASJACEh9GNGZ81m7T6fsbxx") via Dialer { address: "/ip4/192.168.1.108/tcp/42677/p2p/12D3KooWCNL5YR6qBW5Xfa3R8tye4ASJACEh9GNGZ81m7T6fsbxx", role_override: Dialer }
peer 12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN
peer 12D3KooWCNL5YR6qBW5Xfa3R8tye4ASJACEh9GNGZ81m7T6fsbxx
```

详细文档: http://www.libp2p.net.cn/topic/12

