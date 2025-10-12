#!/bin/bash

# ChatGPT-RS API 快速测试脚本（使用 curl）

API_URL="http://localhost:6969"
# 服务器默认使用 http://127.0.0.1:1082 作为代理
# 如果需要覆盖，设置 CUSTOM_PROXY 变量
CUSTOM_PROXY="http://127.0.0.1:1082"  # 例如: "http://custom-proxy:8080"

echo "======================================"
echo "ChatGPT-RS API 快速测试"
echo "======================================"

if [ -n "$CUSTOM_PROXY" ]; then
    echo "使用自定义代理: $CUSTOM_PROXY"
else
    echo "使用服务器默认代理: http://127.0.0.1:1082"
fi

# 测试 1: 简单非流式对话
echo -e "\n[测试 1] 简单对话（非流式）"

# 构建 JSON payload
if [ -n "$CUSTOM_PROXY" ]; then
    PAYLOAD="{\"messages\": [{\"role\": \"user\", \"content\": \"Hello! 请用一句话介绍自己\"}], \"stream\": false, \"proxy\": \"${CUSTOM_PROXY}\"}"
else
    PAYLOAD="{\"messages\": [{\"role\": \"user\", \"content\": \"Hello! 请用一句话介绍自己\"}], \"stream\": false}"
fi

RESPONSE=$(curl -s -X POST "${API_URL}/v1/chat/completions" \
  -H "Content-Type: application/json" \
  -d "$PAYLOAD")

echo "响应:"
echo "$RESPONSE" | jq '.'

# 提取 conversation_id
CONV_ID=$(echo "$RESPONSE" | jq -r '.id')
echo -e "\n会话 ID: $CONV_ID"

# 测试 2: 连续对话
if [ "$CONV_ID" != "null" ] && [ -n "$CONV_ID" ]; then
    echo -e "\n[测试 2] 连续对话（包含对话历史）"
    
    # 提取第一轮的助手回复
    ASSISTANT_REPLY=$(echo "$RESPONSE" | jq -r '.choices[0].message.content')
    
    # 第二轮：告诉名字
    echo "第二轮: 告诉助手名字..."
    if [ -n "$CUSTOM_PROXY" ]; then
        PAYLOAD="{\"conversation_id\": \"${CONV_ID}\", \"messages\": [{\"role\": \"user\", \"content\": \"Hello! 请用一句话介绍自己\"}, {\"role\": \"assistant\", \"content\": $(echo "$ASSISTANT_REPLY" | jq -Rs .)}, {\"role\": \"user\", \"content\": \"我的名字是 Alice\"}], \"stream\": false, \"proxy\": \"${CUSTOM_PROXY}\"}"
    else
        PAYLOAD="{\"conversation_id\": \"${CONV_ID}\", \"messages\": [{\"role\": \"user\", \"content\": \"Hello! 请用一句话介绍自己\"}, {\"role\": \"assistant\", \"content\": $(echo "$ASSISTANT_REPLY" | jq -Rs .)}, {\"role\": \"user\", \"content\": \"我的名字是 Alice\"}], \"stream\": false}"
    fi
    
    RESPONSE2=$(curl -s -X POST "${API_URL}/v1/chat/completions" \
      -H "Content-Type: application/json" \
      -d "$PAYLOAD")
    
    echo "$RESPONSE2" | jq '.'
    ASSISTANT_REPLY2=$(echo "$RESPONSE2" | jq -r '.choices[0].message.content')
    
    # 第三轮：测试是否记住名字
    echo -e "\n第三轮: 测试是否记住名字..."
    if [ -n "$CUSTOM_PROXY" ]; then
        PAYLOAD="{\"conversation_id\": \"${CONV_ID}\", \"messages\": [{\"role\": \"user\", \"content\": \"Hello! 请用一句话介绍自己\"}, {\"role\": \"assistant\", \"content\": $(echo "$ASSISTANT_REPLY" | jq -Rs .)}, {\"role\": \"user\", \"content\": \"我的名字是 Alice\"}, {\"role\": \"assistant\", \"content\": $(echo "$ASSISTANT_REPLY2" | jq -Rs .)}, {\"role\": \"user\", \"content\": \"你还记得我的名字吗？\"}], \"stream\": false, \"proxy\": \"${CUSTOM_PROXY}\"}"
    else
        PAYLOAD="{\"conversation_id\": \"${CONV_ID}\", \"messages\": [{\"role\": \"user\", \"content\": \"Hello! 请用一句话介绍自己\"}, {\"role\": \"assistant\", \"content\": $(echo "$ASSISTANT_REPLY" | jq -Rs .)}, {\"role\": \"user\", \"content\": \"我的名字是 Alice\"}, {\"role\": \"assistant\", \"content\": $(echo "$ASSISTANT_REPLY2" | jq -Rs .)}, {\"role\": \"user\", \"content\": \"你还记得我的名字吗？\"}], \"stream\": false}"
    fi
    
    curl -s -X POST "${API_URL}/v1/chat/completions" \
      -H "Content-Type: application/json" \
      -d "$PAYLOAD" | jq '.'
fi

# 测试 3: 流式响应
echo -e "\n[测试 3] 流式响应"
echo "请求..."
echo -e "响应（实时）:\n"

if [ -n "$CUSTOM_PROXY" ]; then
    PAYLOAD="{\"messages\": [{\"role\": \"user\", \"content\": \"数到 5\"}], \"stream\": true, \"proxy\": \"${CUSTOM_PROXY}\"}"
else
    PAYLOAD="{\"messages\": [{\"role\": \"user\", \"content\": \"数到 5\"}], \"stream\": true}"
fi

curl -s -X POST "${API_URL}/v1/chat/completions" \
  -H "Content-Type: application/json" \
  -d "$PAYLOAD"

# 测试 4: 错误处理
echo -e "\n\n[测试 4] 错误处理（空消息）"
curl -s -X POST "${API_URL}/v1/chat/completions" \
  -H "Content-Type: application/json" \
  -d '{"messages": []}' | jq '.'

echo -e "\n======================================"
echo "测试完成！"
echo "======================================"
