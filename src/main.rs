use reqwest::blocking::Client;

fn main() {
    let client = Client::new();

    let nickname = std::env::args().nth(1).expect("Please provide a nickname");
    let the_rest = std::env::args().skip(2);

    let mut request_body = json::parse(
        r#"[
        {
            "operationName": "FilterableVideoTower_Videos",
            "variables": {
                "limit": 30,
                "channelOwnerLogin": "nickname",
                "broadcastType": "ARCHIVE",
                "videoSort": "TIME"
            },
            "extensions": {
                "persistedQuery": {
                    "version": 1,
                    "sha256Hash": "a937f1d22e269e39a03b509f65a7490f9fc247d7f83d6ac1421523e3b68042cb"
                }
            }
        }
    ]"#,
    )
    .unwrap();

    request_body["variables"]["channelOwnerLogin"] = json::JsonValue::String(nickname.to_string());

    let response = {
        let response = client
            .post("https://gql.twitch.tv/gql")
            .header("Client-Id", "kimne78kx3ncx6brgo4mv6wki5h1ko")
            .body(
                r#"[
                {
                    "operationName": "FilterableVideoTower_Videos",
                    "variables": {
                        "limit": 30,
                        "channelOwnerLogin": "{nickname}",
                        "broadcastType": "ARCHIVE",
                        "videoSort": "TIME"
                    },
                    "extensions": {
                        "persistedQuery": {
                            "version": 1,
                            "sha256Hash": "a937f1d22e269e39a03b509f65a7490f9fc247d7f83d6ac1421523e3b68042cb"
                        }
                    }
                }
            ]"#
                .replace("{nickname}", &nickname)
            )
            .send()
            .unwrap();
        let body = response.text().unwrap();
        json::parse(&body).unwrap()
    };

    let videos_data = response[0]["data"]["user"]["videos"]["edges"]
        .members()
        .map(|video| {
            let game = video["node"]["game"]["name"].as_str().unwrap();
            let title = video["node"]["title"].as_str().unwrap();
            let streamer = video["node"]["owner"]["login"].as_str().unwrap();
            let url = format!(
                "https://www.twitch.tv/videos/{}",
                video["node"]["id"].as_str().unwrap()
            );
            let thumbnail = video["node"]["previewThumbnailURL"].as_str().unwrap();
            (game, title, streamer, url, thumbnail)
        });
    println!("what do you want to do?");
    println!("1. print videos");
    println!("2. watch last video");
    println!("3. exit");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();
    match input {
        "1" => {
            for (index, (game, title, streamer, url, thumbnail)) in
                videos_data.clone().enumerate().rev()
            {
                println!(
                    "[{}] {} - {} - {} - {} - {}",
                    index, game, title, streamer, url, thumbnail
                );
            }
            println!("select video: ");
            let mut video = String::new();
            std::io::stdin().read_line(&mut video).unwrap();
            let video: usize = video.trim().parse().unwrap();
            let mut videos_data = videos_data;
            let url = videos_data.nth(video).unwrap().3;
            if let Ok(fork::Fork::Child) = fork::daemon(false, false) {
                std::process::Command::new("mpv")
                    .arg(&url)
                    .args(the_rest)
                    .output()
                    .unwrap();
            }
            println!("streamlink {} 480p --player=mpv", url);
        }
        "2" => {
            let last_video = videos_data.rev().last().unwrap();
            let (game, title, streamer, url, thumbnail) = last_video;
            println!(
                "{} - {} - {} - {} - {}",
                game, title, streamer, url, thumbnail
            );
            if let Ok(fork::Fork::Child) = fork::daemon(false, false) {
                std::process::Command::new("mpv")
                    .arg(&url)
                    .args(the_rest)
                    .output()
                    .unwrap();
                println!("streamlink {} 480p --player=mpv", url);
            }
        }
        "3" => {
            println!("bye");
        }
        _ => {
            println!("invalid input");
        }
    }
}
