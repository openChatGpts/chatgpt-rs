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


### API服务器

启动API服务器:

```bash
cargo run --bin api_server 6969
```

使用API:

```bash
curl -X POST http://localhost:6969/conversation \
  -H "Content-Type: application/json" \
  -d '{
    "proxy": "http://127.0.0.1:1082",
    "message": "Hello, how are you?"
  }'
```

### 作为库使用

```rust
use chatgpt_rs::client::ChatGptClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ChatGptClient::new(Some("http://127.0.0.1:1082")).await?;
    let response = client.ask_question("Hello!").await?;
    println!("Response: {}", response);
    Ok(())
}
```

## API端点

### POST /conversation

发送消息到ChatGPT并获取响应。

**请求体:**
```json
{
  "proxy": "http://127.0.0.1:1082",
  "message": "你的问题",
  "image": null  // 可选，base64编码的图片
}
```

**响应:**
```json
{
  "status": "success",
  "result": "ChatGPT的回复",
  "error": null
}
```

## 配置

### 代理设置

支持HTTP/HTTPS代理格式:
- `127.0.0.1:8080`
- `http://127.0.0.1:8080`
- `http://user:pass@127.0.0.1:8080`


### 检查代码

```bash
cargo clippy
cargo fmt
```

## 许可证

Apache-2.0

## 贡献

欢迎提交Issue和Pull Request!