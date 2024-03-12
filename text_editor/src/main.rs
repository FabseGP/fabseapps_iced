use iced::executor;
use iced::highlighter::{self, Highlighter};
use iced::keyboard;
use iced::theme;
use iced::widget::{
    button, column, container, horizontal_space, pick_list, row, text, text_editor, tooltip,
};
use iced::window;
use iced::{Application, Command, Element, Font, Settings, Subscription, Theme};

use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

fn main() -> iced::Result {
    Editor::run(Settings {
        default_font: Font::MONOSPACE,
        fonts: vec![include_bytes!("../fonts/editor-icons.ttf")
            .as_slice()
            .into()],
        ..Settings::default()
    })
}

struct Editor {
    path: Option<PathBuf>,
    content: text_editor::Content,
    error: Option<Error>,
    app_theme: Theme,
    highlighter_theme: highlighter::Theme,
    is_dirty: bool,
    scale: f64,
}

#[derive(Debug, Clone)]
enum Error {
    DialogClosed,
    IOFailed(io::ErrorKind),
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    New,
    Open,
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
    Save,
    FileSaved(Result<PathBuf, Error>),
    HighlighterThemeSelected(highlighter::Theme),
    AppThemeSelected(Theme),
    Exit,
    ZoomIn,
    ZoomOut,
}

impl Application for Editor {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                path: None,
                content: text_editor::Content::new(),
                error: None,
                highlighter_theme: highlighter::Theme::SolarizedDark,
                app_theme: Theme::CatppuccinMocha,
                is_dirty: true,
                scale: 1.0,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Fabseediter")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Edit(action) => {
                self.is_dirty = self.is_dirty || action.is_edit();
                self.error = None;
                self.content.perform(action);
                Command::none()
            }
            Message::New => {
                self.path = None;
                self.content = text_editor::Content::new();
                self.is_dirty = true;
                Command::none()
            }
            Message::Open => Command::perform(pick_file(), Message::FileOpened),
            Message::FileOpened(Ok((path, content))) => {
                self.path = Some(path);
                self.content = text_editor::Content::with_text(&content);
                self.is_dirty = false;
                Command::none()
            }
            Message::Save => {
                let text = self.content.text();
                Command::perform(save_file(self.path.clone(), text), Message::FileSaved)
            }
            Message::FileSaved(Ok(path)) => {
                self.path = Some(path);
                self.is_dirty = false;
                Command::none()
            }
            Message::FileSaved(Err(error)) => {
                self.error = Some(error);
                Command::none()
            }
            Message::FileOpened(Err(error)) => {
                self.error = Some(error);
                Command::none()
            }
            Message::HighlighterThemeSelected(theme) => {
                self.highlighter_theme = theme;
                Command::none()
            }
            Message::AppThemeSelected(theme) => {
                self.app_theme = theme;
                Command::none()
            }
            Message::Exit => window::close(window::Id::MAIN),
            Message::ZoomIn => {
                self.scale = self.scale * 1.1;
                Command::none()
            }
            Message::ZoomOut => {
                self.scale = self.scale * 0.9;
                Command::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        keyboard::on_key_press(|key_code, modifiers| match key_code.as_ref() {
            keyboard::Key::Character("n") if modifiers.command() => Some(Message::New),
            keyboard::Key::Character("s") if modifiers.command() => Some(Message::Save),
            keyboard::Key::Character("w") if modifiers.command() => Some(Message::Exit),
            keyboard::Key::Character("+") if modifiers.command() => Some(Message::ZoomIn),
            keyboard::Key::Character("-") if modifiers.command() => Some(Message::ZoomOut),
            _ => None,
        })
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let controls = row![
            action(new_icon(), "New file", Some(Message::New)),
            action(open_icon(), "Open file", Some(Message::Open)),
            action(
                save_icon(),
                "Save file",
                self.is_dirty.then_some(Message::Save)
            ),
            horizontal_space(),
            pick_list(
                highlighter::Theme::ALL,
                Some(self.highlighter_theme),
                Message::HighlighterThemeSelected
            ),
            pick_list(
                Theme::ALL,
                Some(self.app_theme.clone()),
                Message::AppThemeSelected
            )
        ]
        .spacing(10);
        let input = text_editor(&self.content)
            .on_action(Message::Edit)
            .highlight::<Highlighter>(
                highlighter::Settings {
                    theme: self.highlighter_theme,
                    extension: self
                        .path
                        .as_ref()
                        .and_then(|path| path.extension()?.to_str())
                        .unwrap_or("rs")
                        .to_string(),
                },
                |highligth, _theme| highligth.to_format(),
            );
        let status_bar = {
            let status = if let Some(Error::IOFailed(error)) = self.error.as_ref() {
                text(error.to_string())
            } else {
                match self.path.as_deref().and_then(Path::to_str) {
                    Some(path) => text(path).size(14),
                    None => text("New file"),
                }
            };
            let position = {
                let (line, column) = self.content.cursor_position();
                text(format!("{}:{}", line + 1, column + 1))
            };
            row![status, horizontal_space(), position]
        };
        container(column![controls, input, status_bar].spacing(10))
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

fn action<'a>(
    content: Element<'a, Message>,
    label: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let is_disabled = on_press.is_none();
    tooltip(
        button(container(content).width(30).center_x())
            .on_press_maybe(on_press)
            .padding([5, 10])
            .style(if is_disabled {
                theme::Button::Secondary
            } else {
                theme::Button::Primary
            }),
        label,
        tooltip::Position::FollowCursor,
    )
    .style(theme::Container::Box)
    .into()
}

fn new_icon<'a>() -> Element<'a, Message> {
    icon('\u{E800}')
}

fn open_icon<'a>() -> Element<'a, Message> {
    icon('\u{F115}')
}

fn save_icon<'a>() -> Element<'a, Message> {
    icon('\u{E801}')
}

fn icon<'a>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("editor-icons");
    text(codepoint).font(ICON_FONT).into()
}

async fn pick_file() -> Result<(PathBuf, Arc<String>), Error> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Choose a text file...")
        //  .add_filter("text", &["txt", "rs", "c", "cpp", "md", "h", "sh", "toml", "yaml", "config"])
        .pick_file()
        .await
        .ok_or(Error::DialogClosed)?;

    load_file(handle.path().to_owned()).await
}

async fn load_file(path: PathBuf) -> Result<(PathBuf, Arc<String>), Error> {
    let contents = tokio::fs::read_to_string(&path)
        .await
        .map(Arc::new)
        .map_err(|error| error.kind())
        .map_err(Error::IOFailed)?;

    Ok((path, contents))
}

async fn save_file(path: Option<PathBuf>, text: String) -> Result<PathBuf, Error> {
    let path = if let Some(path) = path {
        path
    } else {
        rfd::AsyncFileDialog::new()
            .set_title("Choose a file name...")
            .save_file()
            .await
            .ok_or(Error::DialogClosed)
            .map(|handle| handle.path().to_owned())?
    };
    tokio::fs::write(&path, text)
        .await
        .map_err(|error| Error::IOFailed(error.kind()))?;

    Ok(path)
}
