# Ntrim Environment

设置环境变量，可以改变程序运行的相关参数。

## 语法

```shell
IS_NT_IPV6=1 ./ntrim
```

上述操作，将会启用 IPv6 的连接与`trpc`服务器。

# 通用环境变量

| 参数名                  | 说明                         | 默认值              |
|----------------------|----------------------------|------------------|
| IS_NT_IPV6           | 是否启用 trpc IPv6             | 0                |
| NT_SEND_QUEUE_SIZE   | trpc协议发包队列大小               | 32               |
| HEARTBEAT_INTERVAL   | 标准心跳间隔时间(秒)                | 270              |
| AUTO_RECONNECT       | trpc自动重连                   | 1                |
| RECONNECT_INTERVAL   | trpc自动重连间隔(秒)              | 5                |
| AUTO_REFRESH_SESSION | 自动刷新质押的会话                  | 1                |
| REFRESH_ADVANCE_TIME | 自动会话刷新时间提前(秒)              | 60 * 60 * 24 * 1 |
| SQL_MAX_CONNECTIONS  | 数据库最大连接数                   | 5                |
| ENABLE_SIGN_PROXY    | 是否允许签名请求自动走代理              | 0                |
| IMM_REFRESH_CACHE    | 是否上线成功立即刷新群列表/群成员列表/好友列表缓存 | 1                |
| PING_PONG            | 自回复测试                      | 1                |
| BDH_CHUNK_SIZE       | 资源上传分片大小                   | 1024 * 1024      |

### HEARTBEAT_INTERVAL

默认要求的心跳的心跳270秒，如果大于该时间，可能导致掉线！

### AUTO_REFRESH_SESSION

> `AUTO_REFRESH_SESSION` 默认为开启，质押的会话会在刷新过后的一个月后过期。
> 
> 开启自动刷新可以保证长时间挂机不会掉线，但是会增加封号风险。

### REFRESH_ADVANCE_TIME

`REFRESH_ADVANCE_TIME` 即会在会话过期提前**n**秒刷新会话。

会话刷新即，会话过期前会自动刷新会话，保证不掉线。

#### 使用质押会话模式操作

该模式提供一上线就自动刷新会话的操作，`-i`(`--immediate-refresh`)为**true**的时立即刷新：

```shell
.\ntrim.exe -c [配置文件路径] session -s [质押会话路径] -i true
```

# 缓存及安全策略

| 参数名            | 说明                                                      | 默认值          |
|----------------|---------------------------------------------------------|--------------|
| ENABLE_VOTE    | 是否允许发送点赞包                                               | 1            |
| UID_CACHE_MODE | UID缓存模式(`REVALIDATE`: 需要时缓存, `FULL`: 全量缓存, `NONE`: 不缓存) | `REVALIDATE` |

### UID_CACHE_MODE

- `UID_CACHE_MODE` 为`REVALIDATE`时，会在需要时缓存UID，以避免不必要的缓存。
- `UID_CACHE_MODE` 为`FULL`时，会全量缓存UID，缓存好友列表所有的UID。
- `UID_CACHE_MODE` 为`NONE`时，不会缓存UID，每次都从**数据库获取**或者**发包获取**。

> 如果开启了数据库功能，建议设置为`NONE`避免额外内存开销。

# OneBot环境变量

| 参数名                 | 说明                        | 默认值    |
|---------------------|---------------------------|--------|
| MESSAGE_POST_FORMAT | 消息推送格式(`string`, `array`) | string |

# 开发者环境变量

| 参数名                     | 说明         | 默认值  |
|-------------------------|------------|------|
| RUST_LOG                | 日志级别       | info |
| ENABLE_PRINT_CODEC_LOG  | 是否打印编解码器日志 | 0    |
| ENABLE_PRINT_PUSHPARAMS | 是否打印推送参数   | 0    |

### ENABLE_PRINT_PUSHPARAMS

默认开启，可以在日志中查看推送的参数，例如当前在线设备数量还有在线哪些设备，这些数据不会被缓存！
