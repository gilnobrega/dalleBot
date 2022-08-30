use core::time;
use std::thread;
use reqwest::{
    self,
    header::{AUTHORIZATION, CONTENT_TYPE},
};
use serde_json::{json, Value};

static DALLE_API_URL_TASKS: &str = "https://labs.openai.com/api/labs/tasks";
static DALLE_API_URL_LOGIN: &str = "https://labs.openai.com/api/labs/auth/login";

fn inpainting() {}

pub async fn text2img(caption: &str, dalle_token: &str) -> Result<Value, ()> {
    let prompt = json!({
        "caption": caption,
        "batch_size": 4
    });

    let resp = get_task_response(dalle_token, "text2im", &prompt).await;

    resp
}

async fn get_task_response<'a>(
    dalle_token: &'a str,
    task_type: &'a str,
    prompt: &'a Value,
) -> Result<Value, ()> {
    let body = json!({
        "task_type": task_type,
        "prompt": prompt
    });

    let client = reqwest::Client::new();
    let resp_string = client
        .post(DALLE_API_URL_TASKS)
        .header(AUTHORIZATION, format!("Bearer {}", dalle_token))
        .header(CONTENT_TYPE, "application/json")
        .json(&body)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let resp_json: Value = serde_json::from_str(&resp_string).unwrap();

    let id = &resp_json["id"].to_string().replace("\"", "");
    println!("Task created with id {}", &id);

    let one_second = time::Duration::from_millis(1000);

    let final_data = loop {
        thread::sleep(one_second);

        let url = format!("https://labs.openai.com/api/labs/tasks/{}", &id);
        println!("{}", url);

        let resp_string = client
            .get(url)
            .header(AUTHORIZATION, format!("Bearer {}", dalle_token))
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let resp_json: Value = match serde_json::from_str(&resp_string) {
            Ok(val) => val,
            Err(_) => {
                continue;
            }
        };
        println!("{:?}", &resp_json);

        if resp_json["status"] == "succeeded" {
            break Ok(resp_json);
        }

        if resp_json["status"] != "pending" {
            break Err(());
        }
    };

    println!("Final Response is {:?}", final_data);

    final_data
}

pub async fn get_response_image_urls(response: &Value) -> Vec<String> {
    let mut vec = Vec::new();

    let array = response["generations"].as_object().unwrap()["data"]
        .as_array()
        .unwrap();

    println!("Array is {:?}", array);

    for image in array {
        println!("Image is {:?}", image);

        let url = image["generation"]["image_path"]
            .to_string()
            .replace("\"", "");

        vec.push(url);
    }

    vec
}

pub async fn get_credits(dalle_login_token: &str) -> Result<Option<u64>, ()> {
    let client = reqwest::Client::new();

    let resp_string = client
    .post(DALLE_API_URL_LOGIN)
    .header(AUTHORIZATION, format!("Bearer {}", dalle_login_token))
    .header(CONTENT_TYPE, "application/json")
    .json(&json!({}))
    .send()
    .await
    .unwrap()
    .text()
    .await
    .unwrap();

    println!("{:?}", resp_string);

    let resp_json: Value = match serde_json::from_str(&resp_string) {
        Ok(val) => val,
        Err(_) => return Err(())
    };

    println!("{:?}", resp_json);

    let credits =resp_json["billing_info"]["aggregate_credits"].as_u64();

    Ok(credits)
}
