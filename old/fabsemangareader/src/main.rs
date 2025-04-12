use iced::widget::{
    button, column, container, horizontal_space, image, pick_list, row, text_input,
};
use iced::{
    advanced::Application, Alignment, Command, Element, Font, Settings, Subscription, Theme,
};
use iced::{executor, keyboard, window};

use notify_rust::Notification;

pub fn main() -> iced::Result {
    Mangareader::run(Settings {
        default_font: Font::MONOSPACE,
        fonts: vec![include_bytes!("../fonts/icons.ttf").as_slice().into()],
        ..Settings::default()
    })
}

struct Mangareader {
    app_theme: Theme,
    scale: f64,
    image: image::Handle,
    user_input: String,
}

#[derive(Debug, Clone)]
enum Message {
    Next,
    InputChanged(String),
    AppThemeSelected(Theme),
    ExitApp,
    ZoomIn,
    ZoomOut,
    //  ImageFetched(Result<()>),
}

impl Application for Mangareader {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();
    type Renderer = iced::Renderer;

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                app_theme: Theme::CatppuccinMocha,
                image: image::Handle::from_path("public/natsu_100yearsquest.jpg"),
                scale: 1.0,
                user_input: "".to_string(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Fabseediter")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::AppThemeSelected(theme) => {
                self.app_theme = theme;
                Command::none()
            }
            //   Message::Next => Command::perform(fetch_image(), Message::ImageFetched),
            Message::Next => Command::none(),
            Message::InputChanged(value) => {
                self.user_input = value;
                Command::none()
            }
            Message::ExitApp => {
                Notification::new()
                    .summary("Fabsemanga")
                    .body("🙃 why ye killing me?")
                    .show()
                    .unwrap();
                window::close(window::Id::MAIN)
            }
            Message::ZoomIn => {
                self.scale *= 1.1;
                Command::none()
            }
            Message::ZoomOut => {
                self.scale *= 0.9;
                Command::none()
            } /*  Message::ImageFetched(result) => {
                  match result {
                      Ok(()) => println!("Image fetched successfully"),
                      Err(e) => eprintln!("Error fetching image: {}", e),
                  }
                  Command::none()
              } */
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        keyboard::on_key_press(|key, modifiers| match key.as_ref() {
            keyboard::Key::Character("w") if modifiers.command() => Some(Message::ExitApp),
            keyboard::Key::Character("+") if modifiers.command() => Some(Message::ZoomIn),
            keyboard::Key::Character("-") if modifiers.command() => Some(Message::ZoomOut),
            _ => None,
        })
    }

    fn view(&self) -> Element<Message> {
        let controls = row![
            button("Next image")
                .on_press(Message::Next)
                .style(button::danger),
            horizontal_space(),
            pick_list(
                Theme::ALL,
                Some(self.app_theme.clone()),
                Message::AppThemeSelected
            )
        ]
        .spacing(10);
        let input = text_input("Write a novel", &self.user_input)
            .on_input(Message::InputChanged)
            .size(30)
            .padding(15);
        let image = image::viewer(self.image.clone());
        container(
            column![controls, input, image]
                .spacing(10)
                .align_items(Alignment::Center),
        )
        .padding(10)
        .into()
    }

    fn theme(&self) -> Theme {
        self.app_theme.clone()
    }

    fn scale_factor(&self) -> f64 {
        self.scale
    }
}
