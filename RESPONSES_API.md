# Responses API Documentation

本项目已迁移到 OpenAI 的 Responses API 架构，提供更好的有状态对话管理。

## 🏗️ 新架构

代码已重构为模块化结构：

```
src/api/
├── mod.rs          # 模块导出
├── server.rs       # 路由配置和服务器启动
├── handlers.rs     # 请求处理器
├── state.rs        # 应用状态管理（线程管理）
├── types.rs        # 请求/响应类型定义
└── error.rs        # 错误处理
```

## 📚 API 端点

### 线程管理 (Threads)

#### 1. 创建线程
```bash
POST /v1/threads
Content-Type: application/json

{
  "messages": [
    {"role": "user", "content": "我叫 Alice"}
  ],
  "metadata": {},  // 可选
  "proxy": "http://proxy:port"  // 可选
}
```

响应：
```json
{
  "id": "thread_xxx",
  "object": "thread",
  "created_at": 1234567890,
  "metadata": null
}
```

#### 2. 获取线程
```bash
GET /v1/threads/{thread_id}
```

#### 3. 列出所有线程
```bash
GET /v1/threads
```

响应：
```json
{
  "object": "list",
  "data": [
    {
      "id": "thread_xxx",
      "object": "thread",
      "created_at": 1234567890,
      "metadata": null
    }
  ],
  "has_more": false
}
```

#### 4. 删除线程
```bash
DELETE /v1/threads/{thread_id}
```

### 消息管理 (Messages)

#### 1. 添加消息到线程
```bash
POST /v1/threads/{thread_id}/messages
Content-Type: application/json

{
  "role": "user",
  "content": "你记得我叫什么吗？"
}
```

响应：
```json
{
  "id": "msg_xxx",
  "object": "thread.message",
  "created_at": 1234567890,
  "thread_id": "thread_xxx",
  "role": "user",
  "content": [
    {
      "type": "text",
      "text": {
        "value": "你记得我叫什么吗？",
        "annotations": []
      }
    }
  ]
}
```

#### 2. 列出线程中的所有消息
```bash
GET /v1/threads/{thread_id}/messages
```

响应：
```json
{
  "object": "list",
  "data": [
    {
      "id": "msg_0",
      "object": "thread.message",
      "created_at": 1234567890,
      "thread_id": "thread_xxx",
      "role": "user",
      "content": [...]
    },
    {
      "id": "msg_1",
      "object": "thread.message",
      "created_at": 1234567891,
      "thread_id": "thread_xxx",
      "role": "assistant",
      "content": [...]
    }
  ],
  "has_more": false
}
```

### 响应生成 (Responses)

#### 创建响应（运行助手）
```bash
POST /v1/responses
Content-Type: application/json

{
  "thread_id": "thread_xxx",
  "stream": false,  // 可选，默认 false
  "model": "gpt-4",  // 可选
  "instructions": "..."  // 可选
}
```

非流式响应：
```json
{
  "id": "response_xxx",
  "object": "thread.response",
  "created_at": 1234567890,
  "thread_id": "thread_xxx",
  "status": "completed",
  "model": "gpt-4",
  "usage": {
    "prompt_tokens": 0,
    "completion_tokens": 0,
    "total_tokens": 0
  }
}
```

流式响应（设置 `"stream": true`）：
```
data: {"id":"response_xxx","object":"thread.response.chunk","created_at":1234567890,"thread_id":"thread_xxx","delta":{"role":"assistant","content":"你好"}}

data: {"id":"response_xxx","object":"thread.response.chunk","created_at":1234567890,"thread_id":"thread_xxx","delta":{"content":"，Alice"}}

data: {"id":"response_xxx","object":"thread.response.chunk","created_at":1234567890,"thread_id":"thread_xxx","delta":{}}
```

## 🔄 使用流程

### 完整对话示例

```bash
# 1. 创建线程并添加初始消息
curl -X POST http://localhost:6969/v1/threads \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [
      {"role": "user", "content": "我叫 Alice"}
    ]
  }'

# 响应: {"id": "thread_abc123", ...}

# 2. 生成 AI 响应
curl -X POST http://localhost:6969/v1/responses \
  -H "Content-Type: application/json" \
  -d '{
    "thread_id": "thread_abc123"
  }'

# 3. 查看对话历史
curl http://localhost:6969/v1/threads/thread_abc123/messages

# 4. 继续对话 - 添加新消息
curl -X POST http://localhost:6969/v1/threads/thread_abc123/messages \
  -H "Content-Type: application/json" \
  -d '{
    "role": "user",
    "content": "你记得我叫什么吗？"
  }'

# 5. 再次生成响应
curl -X POST http://localhost:6969/v1/responses \
  -H "Content-Type: application/json" \
  -d '{
    "thread_id": "thread_abc123"
  }'
```


### 新 API (Responses)
```bash
# 服务器管理对话状态
POST /v1/threads/{thread_id}/messages
{"role": "user", "content": "你记得我叫什么吗？"}

POST /v1/responses
{"thread_id": "thread_xxx"}
```

## 🎯 优势

1. **服务器端状态管理**：不需要客户端发送完整的消息历史
2. **更好的上下文维护**：每个线程自动维护完整的对话上下文
3. **符合 OpenAI 标准**：与 OpenAI Assistants API 兼容
4. **清晰的架构**：线程、消息、响应分离明确
5. **支持流式和非流式**：灵活的响应模式

## 🧪 测试

运行测试脚本：

```bash
# 启动服务器
cargo run --bin api_server

# 在另一个终端运行测试
python3 test_responses_api.py
```

## 🔧 其他端点

### 健康检查
```bash
GET /health
```

响应：
```json
{
  "status": "ok",
  "proxy": "none",
  "active_threads": 1,
  "version": "0.1.0"
}
```

### 列出模型
```bash
GET /v1/models
```

响应：
```json
{
  "object": "list",
  "data": [
    {
      "id": "gpt-4",
      "object": "model",
      "created": 1677610602,
      "owned_by": "openai"
    },
    {
      "id": "gpt-4o",
      "object": "model",
      "created": 1715367049,
      "owned_by": "openai"
    }
  ]
}
```
