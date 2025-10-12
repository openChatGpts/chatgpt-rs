#!/usr/bin/env python3
"""
Test script for the new Responses API
This demonstrates how to use threads and responses for stateful conversations
"""

import requests
import json

BASE_URL = "http://localhost:6969"

def print_response(title, response):
    """Pretty print a response"""
    print(f"\n{'='*60}")
    print(f"{title}")
    print(f"{'='*60}")
    print(f"Status: {response.status_code}")
    try:
        print(json.dumps(response.json(), indent=2, ensure_ascii=False))
    except:
        print(response.text)

def test_responses_api():
    """Test the Responses API workflow"""
    
    # 1. 创建一个新线程（Thread）
    print("\n📝 1. Creating a new thread...")
    response = requests.post(f"{BASE_URL}/v1/threads", json={
        "messages": [
            {"role": "user", "content": "我叫 Alice"}
        ]
    })
    print_response("Create Thread Response", response)
    
    if response.status_code != 200:
        print("❌ Failed to create thread")
        return
    
    thread_id = response.json()["id"]
    print(f"\n✅ Thread created: {thread_id}")
    
    # 2. 创建第一个响应（让 AI 回复）
    print("\n🤖 2. Creating first response...")
    response = requests.post(f"{BASE_URL}/v1/responses", json={
        "thread_id": thread_id,
        "stream": False
    })
    print_response("First Response", response)
    
    # 3. 获取线程中的所有消息
    print("\n📜 3. Getting all messages in thread...")
    response = requests.get(f"{BASE_URL}/v1/threads/{thread_id}/messages")
    print_response("Thread Messages", response)
    
    # 4. 添加新消息到线程
    print("\n💬 4. Adding a new message to thread...")
    response = requests.post(f"{BASE_URL}/v1/threads/{thread_id}/messages", json={
        "role": "user",
        "content": "你记得我叫什么吗？"
    })
    print_response("Add Message Response", response)
    
    # 5. 创建第二个响应（继续对话）
    print("\n🤖 5. Creating second response (continuing conversation)...")
    response = requests.post(f"{BASE_URL}/v1/responses", json={
        "thread_id": thread_id,
        "stream": False
    })
    print_response("Second Response", response)
    
    # 6. 再次获取所有消息，查看完整对话
    print("\n📜 6. Getting all messages again...")
    response = requests.get(f"{BASE_URL}/v1/threads/{thread_id}/messages")
    print_response("All Messages", response)
    
    # 7. 列出所有线程
    print("\n📋 7. Listing all threads...")
    response = requests.get(f"{BASE_URL}/v1/threads")
    print_response("All Threads", response)
    
    # 8. 测试流式响应
    print("\n🌊 8. Testing streaming response...")
    response = requests.post(f"{BASE_URL}/v1/threads/{thread_id}/messages", json={
        "role": "user",
        "content": "再说一遍我的名字"
    })
    
    print("\n🌊 Creating streaming response...")
    response = requests.post(f"{BASE_URL}/v1/responses", json={
        "thread_id": thread_id,
        "stream": True
    }, stream=True)
    
    print("Status:", response.status_code)
    print("Streaming response:")
    for line in response.iter_lines():
        if line:
            print(line.decode('utf-8'))
    
    print("\n" + "="*60)
    print("✅ All tests completed!")
    print("="*60)

def test_health():
    """Test health endpoint"""
    print("\n🏥 Testing health endpoint...")
    response = requests.get(f"{BASE_URL}/health")
    print_response("Health Check", response)

if __name__ == "__main__":
    print("""
╔══════════════════════════════════════════════════════════╗
║       ChatGPT-RS Responses API Test Suite               ║
║                                                          ║
║  This tests the new OpenAI-compatible Responses API     ║
║  which provides stateful conversation management        ║
╚══════════════════════════════════════════════════════════╝
    """)
    
    # Test health first
    test_health()
    
    # Run main tests
    test_responses_api()
