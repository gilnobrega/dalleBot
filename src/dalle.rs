use core::time;
use std::thread;

use bytes::Bytes;
use reqwest::{self, header::{CONTENT_TYPE, AUTHORIZATION}, Response}; 
use tokio;
use serde_json::{json, Value};

static DALLE_API_URL: &str = "https://labs.openai.com/api/labs/tasks";

fn inpainting()
{

}

pub async fn text2img(caption: &str, dalle_token: &str) -> Value
{
    let prompt = json!({
        "caption": caption,
        "batch_size": 4
    });

    let resp = get_task_response(dalle_token, "text2im", &prompt);

    resp.await
}

async fn get_task_response(dalle_token: &str, task_type: &str, prompt: &Value) -> Value
{
    let body = json!({
        "task_type": task_type,
        "prompt": prompt
    });
    
    let client = reqwest::Client::new();
    let resp_string = client.post(DALLE_API_URL)
        .header(AUTHORIZATION, format!("Bearer {}", dalle_token))
        .header(CONTENT_TYPE, "application/json")
        .json(&body)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    
    let resp_json:Value = serde_json::from_str(&resp_string).unwrap();

    let id = &resp_json["id"].to_string().replace("\"", "");
    println!("Task created with id {}", &id);


    let one_second = time::Duration::from_millis(1000);

    let final_data = loop 
    {
        thread::sleep(one_second);

        let url = format!("https://labs.openai.com/api/labs/tasks/{}", &id);
        println!("{}", url);

        let resp_string = client.get(url)
        .header(AUTHORIZATION, format!("Bearer {}", dalle_token))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

        let resp_json:Value = match serde_json::from_str(&resp_string) {
            Ok(val) => val,
            Err(err) => {
                continue;
            },
        };
        println!("{:?}", &resp_json);

        if resp_json["status"] == "succeeded"
        {
            break resp_json;
        }
    };

    println!("Final Response is {:?}", final_data);

    final_data

}

pub async fn download_response_image(response: &Value) -> Vec<Bytes>
{
    let mut vec = Vec::new();

    let array = response["generations"].as_object().unwrap()["data"].as_array().unwrap();

    println!("Array is {:?}", array);

    for image in array
    {
        println!("Image is {:?}", image);

        let url = image["generation"]["image_path"].to_string().replace("\"", "");

        println!("Url is {:?}", url);

        //let img_bytes = reqwest::get("https://cdn.openai.com/labs/images/A%20Shiba%20Inu%20dog%20wearing%20a%20beret%20and%20black%20turtleneck.webp?v=1")
        let img_bytes = reqwest::get(url)
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();

        vec.push(img_bytes);
    }

    vec
}