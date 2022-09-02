本文档和代码测试 编写于2022年9月1日。
注明日期是防止以后有人看到这篇文章的时候，libp2p的对应版本变化太大导致不能使用。

1. 操作环境 三台机器（也可以是2台机器）
2. relay 服务器
3. 两台client 机器，其中一台在docker 里面,以后称为A，B

实现目标：docker 里面的机器(A)和外面的 （B）机器互联发送消息。

第一个问题:
A，B 原本是互相不知道 IP以及端口号的。他们如何互联？
答：通过 relay 服务器

第二个问题：
通过relay服务器后，可以关闭relay服务器后，实现他们互相通信吗？
答： 可以？

如何做到以上 过程？
1. relay 会收集他们的信息，并且告知对方 在 外部机器的ip+port
本次实验环境 主要是告诉docker 他在Linux 主机的 端口号和主机的IP。

2. 告诉对方IP，他们如何互联？
dcutr, dcutr协议 首先 会把A，或者B的信息告诉对方，这取决于谁是后来者。

如果B先连接relay，A后连接relay，那么 A就可以去连接B。
问题是 A怎么知道B的存在？ 第3步会解释这个问题。

现在我们假设 A知道了B的存在，A就会dial B，这个dial是通过relay的。

dcutr 此时就会 尝试A，B互联。连接成功 就可以通讯了。

现在的版本dcutr有一点问题，需要修改，
修改文章见： http://www.libp2p.net.cn/topic/7/rust-libp2p-%E4%BD%BF%E7%94%A8relay%E5%81%9A%E6%89%93%E6%B4%9E-%E7%A9%BF%E9%80%8F%E7%9A%84%E6%96%B9%E6%A1%88-%E9%81%87%E5%88%B0%E7%9A%84%E7%B3%BB%E7%BB%9Fbug%E5%8F%8A%E5%85%B6%E8%A7%A3%E5%86%B3%E6%96%B9%E6%A1%88

3. 后来这A，如何知道之前的B存在？
这就是 rendezvous 
当B的时候 连接relay，连接成功后relay告诉他 外部IP+port，
B此时 调用 rendezvous（ relay 服务器的）进行注册。注册成后，
就可以调用 discover 来获取当前有多少 peer注册了 这个 rendezvous。

所以后来人A，也会注册自己并且调用  discover 来获取之前的人。


源代码：https://github.com/libp2p-examples/rust-examples

https://github.com/libp2p-examples/rust-examples/blob/main/examples/05-relay.rs

这是 relay server代码，直接编译就可以。
他里面生成了固定的 peerid，端口是 45678。

https://github.com/libp2p-examples/rust-examples/blob/main/examples/06-hole_punching.rs

这是 打洞的client代码， 里面有 dcutr，rendezvous， 和互相发送消息的自己写的协议。这个协议源代码在 https://github.com/libp2p-examples/rust-examples/tree/main/src
这个协议 就叫做msg吧。 你可以使用别的代替例如:ping，chat里面用的FloodsubEvent::Message。


编译服务器：
```
cargo build --example 05-relay
```

编译 client,注意这里 需要修改 dcutr源代码，修改方法见上面的文档
```
cargo build --example 06-hole_punching
```

运行:
在Windows上运行 relay server

```
PS C:\Users\qiyuan\Desktop\rust-examples-main> .\target\debug\examples\05_relay
opt: Opt { use_ipv6: None, secret_key_seed: 0, port: 45678 }
Local peer id: PeerId("12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN")
Listening on "/ip4/192.168.1.105/tcp/45678"
Listening on "/ip4/127.0.0.1/tcp/45678"
```

这里relay 服务器地址就是:/ip4/192.168.1.105/tcp/45678

运行B 客户端 Linux 机器
```
klang@klang-LAPKC71E:~/rust-examples$ ./target/debug/examples/06-hole_punching --relay-address /ip4/192.168.1.105/tcp/45678
Local peer id: PeerId("12D3KooWCNL5YR6qBW5Xfa3R8tye4ASJACEh9GNGZ81m7T6fsbxx")
Listening on "/ip4/127.0.0.1/tcp/42677"
Listening on "/ip4/192.168.1.108/tcp/42677"
Relay told us our public address: "/ip4/192.168.1.108/tcp/42677"
Registered for namespace 'rendezvous' at rendezvous point 12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN for the next 7200 seconds
Discovered peer 12D3KooWCNL5YR6qBW5Xfa3R8tye4ASJACEh9GNGZ81m7T6fsbxx at /ip4/192.168.1.108/tcp/42677

```

服务告诉我的 

运行A 在一台Linux docker里面

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

他发现了 B和他自己。并且连接到B成功了。

断开 relay 发消息测试：
```
disconnect PeerId("12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN") by Dialer { address: "/ip4/192.168.1.105/tcp/45678", role_override: Dialer }

```

收到断开消息。
在A，或者B 敲键盘回车。
 例如：A敲
```
1111
```

B
```
disconnect PeerId("12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN") by Dialer { address: "/ip4/192.168.1.105/tcp/45678", role_override: Dialer }
PeerId PeerId("12D3KooWHrxvWRcsQxY9WmMxWwcKMAtsJZw3gDj6aDDqisnbihK8"),ConnId ConnectionId(2)
Event { peer: PeerId("12D3KooWHrxvWRcsQxY9WmMxWwcKMAtsJZw3gDj6aDDqisnbihK8"), result: MsgContent { data: [49, 49, 49, 49] } }

````


