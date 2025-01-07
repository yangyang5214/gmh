use std::process::Command;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use dotenv::dotenv;
use std::{env, process};
use std::path::Path;

#[derive(Serialize, Debug)]
struct DeepSeekRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

#[derive(Serialize, Debug)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize, Debug)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
    prompt_cache_hit_tokens: u32,
    prompt_cache_miss_tokens: u32,
}


#[derive(Deserialize, Debug)]
struct DeepSeekResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<Choice>,
    usage: Usage,
    system_fingerprint: String,
}
#[derive(Deserialize, Debug)]
struct Choice {
    index: u32,
    message: MessageResponse,
    logprobs: Option<serde_json::Value>, // 可以是 null，所以用 Option
    finish_reason: String,
}

#[derive(Deserialize, Debug)]
struct MessageResponse {
    #[allow(dead_code)]
    role: String,
    content: String,
}

async fn get_git_diff() -> Result<String, String> {
    let output = Command::new("git")
        .arg("diff")
        .arg("--cached")
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

async fn generate_commit_message(diff: &str) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set in .env file");
    let client = Client::new();

    let request_body = DeepSeekRequest {
        model: "deepseek-chat".to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: "You are a helpful assistant to great a short git commit message".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: diff.to_string(),
            },
        ],
        stream: false,
    };

    // let json_body = serde_json::to_string_pretty(&request_body).expect("Failed to serialize request body");
    // println!("Request body (JSON):\n{}", json_body);

    let response = client
        .post("https://api.deepseek.com/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;


    let response_body: DeepSeekResponse = response.json().await?;

    // 提取助手的回复
    if let Some(choice) = response_body.choices.first() {
        Ok(choice.message.content.clone())
    } else {
        Err("No response from DeepSeek".into())
    }
}

async fn commit_changes(commit_message: &str) -> Result<(), String> {
    let status = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(commit_message)
        .status()
        .map_err(|e| e.to_string())?;

    if status.success() {
        Ok(())
    } else {
        Err("Failed to commit changes".to_string())
    }
}

fn is_git_repository() -> bool {
    Path::new(".git").exists()
}


#[tokio::main]
async fn main() {
    if !is_git_repository() {
        eprintln!("Current directory is not a Git repository.");
        return;
    }

    dotenv().ok(); // 加载 .env 文件

    // 获取 git diff
    let diff = match get_git_diff().await {
        Ok(diff) => diff,
        Err(err) => {
            eprintln!("Error getting git diff: {}", err);
            return;
        }
    };

    if diff.is_empty() {
        println!("No changes detected.");
        return;
    }

    // 生成 commit 消息
    let commit_message = match generate_commit_message(&diff).await {
        Ok(message) => message,
        Err(err) => {
            eprintln!("Error generating commit message: {}", err);
            return;
        }
    };

    println!("Generated commit message:\n{}", commit_message);

    println!("Do you want to commit these changes? (y/n)");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("Failed to read input");

    if input.trim().to_lowercase() == "y" {
        if let Err(err) = commit_changes(&commit_message).await {
            eprintln!("Error committing changes: {}", err);
        } else {
            println!("Changes committed successfully.");
        }
    } else {
        println!("Commit canceled.");
    }
}