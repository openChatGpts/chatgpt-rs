# ChatGPT Rust Client
一个用Rust编写的ChatGPT逆向工程客户端
## 项目结构

```
src/
├── api/           # API服务器模块
├── client/        # ChatGPT客户端核心
├── crypto/        # 加密和挑战解决
├── network/       # 网络相关功能
├── utils/         # 工具函数和错误处理
├── vm/            # JavaScript VM执行器
└── lib.rs         # 库入口
```

构建项目:

```bash
git clone 
cd chatgpt-rs
cargo build --release
```

# ChatGPT-RS API 使用说明

## 概述

这是一个兼容 OpenAI API 的 ChatGPT 接口服务，支持流式和非流式响应，以及连续对话。

## 启动服务器

### 基本启动

```bash
cargo run --bin api_server
```

服务器默认配置：
- 监听地址: `0.0.0.0:6969`
- 默认代理: `http://127.0.0.1:1082`

### 命令行参数

```bash
# 查看帮助
cargo run --bin api_server -- --help

# 自定义端口
cargo run --bin api_server -- --port 8080

# 自定义代理
cargo run --bin api_server -- --proxy http://proxy.example.com:8080

# 自定义主机和端口
cargo run --bin api_server -- --host 127.0.0.1 --port 8080

# 不使用代理
cargo run --bin api_server -- --no-proxy

# 组合使用
cargo run --bin api_server -- --host 127.0.0.1 --port 8080 --proxy http://localhost:7890
```

### 环境变量配置（可选）

环境变量会覆盖命令行参数：

```bash
# 设置监听地址
export API_HOST=127.0.0.1

# 设置监听端口
export API_PORT=8080

# 设置默认代理
export DEFAULT_PROXY=http://username:password@proxy.example.com:8080

# 启动服务器
cargo run --bin api_server
```

## API 端点

### OpenAI 兼容端点

**POST** `/v1/chat/completions`

这是对话接口，完全兼容 OpenAI 的 API 格式。

#### 请求格式

```json
{
  "model": "gpt-4",
  "messages": [
    {
      "role": "user",
      "content": "你好，请介绍一下自己"
    }
  ],
  "stream": false,
  "conversation_id": "optional-conversation-id",
  "proxy": "http://proxy.example.com:8080"
}
```

#### 参数说明

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `model` | string | 否 | 模型名称（目前会被忽略，使用 ChatGPT 的默认模型） |
| `messages` | array | 是 | 对话消息数组 |
| `messages[].role` | string | 是 | 角色：`user` 或 `assistant` |
| `messages[].content` | string | 是 | 消息内容 |
| `stream` | boolean | 否 | 是否使用流式响应，默认 `false` |
| `conversation_id` | string | 否 | 会话 ID，用于连续对话。如果不提供，会创建新会话 |
| `proxy` | string | 否 | 代理服务器地址，覆盖默认代理。如果不提供，使用服务器的默认代理 |
| `temperature` | float | 否 | 温度参数（暂不支持，保留用于兼容） |
| `max_tokens` | int | 否 | 最大令牌数（暂不支持，保留用于兼容） |

**注意：** 如果请求中不包含 `proxy` 参数，将使用服务器启动时配置的默认代理（默认为 `http://127.0.0.1:1082`）。

#### 非流式响应示例

**请求：**

```bash
# 使用默认代理
curl -X POST http://localhost:6969/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [
      {"role": "user", "content": "什么是 Rust 编程语言？"}
    ],
    "stream": false
  }'

# 或指定特定代理
curl -X POST http://localhost:6969/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [
      {"role": "user", "content": "什么是 Rust 编程语言？"}
    ],
    "stream": false,
    "proxy": "http://custom-proxy:8080"
  }'
```

**响应：**

```json
{
  "id": "conv-123e4567-e89b-12d3-a456-426614174000",
  "object": "chat.completion",
  "created": 1697123456,
  "model": "gpt-4",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Rust 是一种系统编程语言..."
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 0,
    "completion_tokens": 0,
    "total_tokens": 0
  }
}
```

#### 流式响应示例

**请求：**

```bash
curl -X POST http://localhost:6969/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [
      {"role": "user", "content": "用一句话介绍 Python"}
    ],
    "stream": true
  }'
```

**响应（Server-Sent Events）：**

```
data: {"id":"conv-123","object":"chat.completion.chunk","created":1697123456,"model":"gpt-4","choices":[{"index":0,"delta":{"role":"assistant","content":"Python"},"finish_reason":null}]}

data: {"id":"conv-123","object":"chat.completion.chunk","created":1697123456,"model":"gpt-4","choices":[{"index":0,"delta":{"content":" 是一种"},"finish_reason":null}]}

...

data: {"id":"conv-123","object":"chat.completion.chunk","created":1697123456,"model":"gpt-4","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}
```

#### 连续对话示例

**第一轮对话：**

```bash
curl -X POST http://localhost:6969/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [
      {"role": "user", "content": "你好"}
    ]
  }'
```

响应会包含一个 `id` 字段，例如 `"conv-123e4567..."`

**第二轮对话（使用相同的 conversation_id）：**

```bash
curl -X POST http://localhost:6969/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "conversation_id": "conv-123e4567-e89b-12d3-a456-426614174000",
    "messages": [
      {"role": "user", "content": "你好"},
      {"role": "assistant", "content": "你好！有什么我可以帮助你的吗？"},
      {"role": "user", "content": "刚才我说了什么？"}
    ]
  }'
```

**注意：** 为了保持对话上下文，需要：
1. 使用相同的 `conversation_id`
2. 在 `messages` 数组中包含之前的对话历史（只需要最后一条用户消息也可以）

## 使用场景

### 场景 1：简单问答

适用于单次问答，不需要保持上下文：

```bash
curl -X POST http://localhost:6969/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [
      {"role": "user", "content": "什么是机器学习？"}
    ]
  }'
```

### 场景 2：连续对话

适用于需要保持对话上下文的场景：

```python
import requests
import json

url = "http://localhost:6969/v1/chat/completions"
conversation_id = None
messages = []

def chat(user_message):
    global conversation_id, messages
    
    messages.append({"role": "user", "content": user_message})
    
    payload = {
        "messages": messages,
        "stream": False
    }
    
    if conversation_id:
        payload["conversation_id"] = conversation_id
    
    response = requests.post(url, json=payload)
    data = response.json()
    
    # 保存 conversation_id
    if not conversation_id:
        conversation_id = data["id"]
    
    # 保存助手的回复
    assistant_message = data["choices"][0]["message"]["content"]
    messages.append({"role": "assistant", "content": assistant_message})
    
    return assistant_message

# 使用示例
print(chat("你好"))
print(chat("刚才我说了什么？"))  # 会记住之前的对话
```

### 场景 3：流式响应

适用于需要实时显示响应的场景（如聊天界面）：

```python
import requests
import json

url = "http://localhost:6969/v1/chat/completions"

payload = {
    "messages": [
        {"role": "user", "content": "写一个 Python 函数来计算斐波那契数列"}
    ],
    "stream": True
}

response = requests.post(url, json=payload, stream=True)

for line in response.iter_lines():
    if line:
        line = line.decode('utf-8')
        if line.startswith('data: '):
            data = json.loads(line[6:])
            delta = data["choices"][0]["delta"]
            if "content" in delta:
                print(delta["content"], end='', flush=True)
```

### 场景 4：使用特定代理

如果某些请求需要使用不同于默认代理的特定代理：

```bash
curl -X POST http://localhost:6969/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "proxy": "http://special-proxy.example.com:8080",
    "messages": [
      {"role": "user", "content": "Hello"}
    ]
  }'
```

如果不指定 `proxy` 参数，将使用服务器启动时配置的默认代理。

## 与 OpenAI 官方 SDK 的集成

可以使用 OpenAI 的官方 SDK，只需修改 base_url：

### Python

```python
from openai import OpenAI

client = OpenAI(
    api_key="dummy-key",  # 随便填一个，不会被使用
    base_url="http://localhost:6969/v1"
)

# 简单对话
response = client.chat.completions.create(
    model="gpt-4",
    messages=[
        {"role": "user", "content": "你好"}
    ]
)
print(response.choices[0].message.content)

# 流式响应
stream = client.chat.completions.create(
    model="gpt-4",
    messages=[
        {"role": "user", "content": "写一个故事"}
    ],
    stream=True
)

for chunk in stream:
    if chunk.choices[0].delta.content:
        print(chunk.choices[0].delta.content, end='')
```

### JavaScript/TypeScript

```typescript
import OpenAI from 'openai';

const client = new OpenAI({
  apiKey: 'dummy-key',
  baseURL: 'http://localhost:6969/v1',
});

// 简单对话
const response = await client.chat.completions.create({
  model: 'gpt-4',
  messages: [
    { role: 'user', content: '你好' }
  ],
});
console.log(response.choices[0].message.content);

// 流式响应
const stream = await client.chat.completions.create({
  model: 'gpt-4',
  messages: [
    { role: 'user', content: '写一个故事' }
  ],
  stream: true,
});

for await (const chunk of stream) {
  process.stdout.write(chunk.choices[0]?.delta?.content || '');
}
```

## 错误处理

API 使用标准的 HTTP 状态码：

- `200` - 成功
- `400` - 请求参数错误
- `401` - 认证失败
- `403` - IP 被标记
- `422` - 数据格式错误
- `500` - 服务器内部错误
- `502` - 网络错误

错误响应格式：

```json
{
  "status": "error",
  "detail": "错误详细信息"
}
```

## 会话管理

- 每个 `conversation_id` 对应一个 `ChatGptClient` 实例
- 客户端实例会在内存中缓存，避免重复创建
- 连续对话时使用相同的 `conversation_id` 可以保持上下文
- 如果不提供 `conversation_id`，每次请求都会创建新的会话

## 性能考虑

1. **连接复用**：相同的 `conversation_id` 会复用同一个客户端实例
2. **并发处理**：服务器使用异步 I/O，可以同时处理多个请求
3. **流式响应**：对于长文本回复，建议使用流式响应以获得更好的用户体验

## 安全建议

1. 在生产环境中，建议在前面加一层反向代理（如 Nginx）
2. 使用 HTTPS 加密通信
3. 限制请求速率，防止滥用
4. 代理服务器凭据应该通过环境变量配置，不要硬编码在代码中



### 连接超时

检查代理设置是否正确：

```bash
# 测试代理连接
curl -x http://your-proxy:port https://chatgpt.com
```

### IP 被标记

如果收到 "Unusual activity" 错误，说明 IP 被 ChatGPT 标记，需要更换代理或 IP。


# 构建发布版本
cargo build --release --bin api_server

# 运行发布版本
./target/release/api_server --port 8080 --proxy http://localhost:7890
```



### 检查代码

```bash
cargo clippy
cargo fmt
```

## 许可证

Apache-2.0

## 贡献

欢迎提交Issue和Pull Request!