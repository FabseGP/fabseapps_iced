use iced::executor;
use iced::widget::{image, text, Image};
use iced::{Application, Command, Element, Settings, Theme};
use serde::{Deserialize, Serialize};

fn main() -> iced::Result {
    MangaReader::run(Settings::default())
}

struct MangaReader {}

#[derive(Clone, Debug)]
enum Message {}

impl Application for MangaReader {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (Self {}, Command::none())
    }

    fn title(&self) -> String {
        String::from("Fabsemangaka")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {}
    }

    fn view(&self) -> Element<'_, Self::Message> {
        // text("Hello there!").into()
        Image::<image::Handle>::new("public/natsu_flame.jpg").into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

async fn chapters() {
    #[derive(Deserialize, Serialize)]
    struct MangadexResponse {
        chapter: Chapter,
    }

    #[derive(Deserialize, Serialize)]
    struct Chapter {
        data: Vec<String>,
    }

    let fetch_entry = async {
        let url = format!(
            "https://api.mangadex.org/at-home/server/{chapter_id}",
            chapter_id = "27cd0902-ad4c-490a-b752-ae032f0503c9"
        );
        reqwest::get(&url).await
    };

    //let data: MangadexResponse = fetch_entry.json().await.unwrap();
}
