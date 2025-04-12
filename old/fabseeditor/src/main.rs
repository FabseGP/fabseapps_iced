use iced::widget::{
    button, column, container, horizontal_space, pick_list, row, text, text_editor, tooltip,
};
use iced::{
    advanced::Application, Alignment, Command, Element, Font, Length, Settings, Subscription, Theme,
};
use iced::{
    executor,
    highlighter::{self, Highlighter},
    keyboard, window,
};

use std::{
    ffi, io,
    path::{Path, PathBuf},
    sync::Arc,
};

pub fn main() -> iced::Result {
    Editor::run(Settings {
        default_font: Font::MONOSPACE,
        fonts: vec![include_bytes!("../fonts/icons.ttf").as_slice().into()],
        ..Settings::default()
    })
}

struct Editor {
    file: Option<PathBuf>,
    content: text_editor::Content,
    highlighter_theme: highlighter::Theme,
    app_theme: Theme,
    is_loading: bool,
    is_dirty: bool,
    scale: f64,
}

#[derive(Debug, Clone)]
enum Message {
    ActionPerformed(text_editor::Action),
    HighlighterThemeSelected(highlighter::Theme),
    AppThemeSelected(Theme),
    NewFile,
    OpenFile,
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
    SaveFile,
    FileSaved(Result<PathBuf, Error>),
    ExitApp,
    ZoomIn,
    ZoomOut,
}

impl Application for Editor {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();
    type Renderer = iced::Renderer;

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                file: None,
                content: text_editor::Content::new(),
                highlighter_theme: highlighter::Theme::SolarizedDark,
                app_theme: Theme::CatppuccinMocha,
                is_loading: false,
                is_dirty: false,
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
            Message::ActionPerformed(action) => {
                self.is_dirty |= action.is_edit();
                self.content.perform(action);
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
            Message::NewFile => {
                if !self.is_loading {
                    self.file = None;
                    self.content = text_editor::Content::new();
                }
                Command::none()
            }
            Message::OpenFile => {
                if self.is_loading {
                    Command::none()
                } else {
                    self.is_loading = true;
                    Command::perform(open_file(), Message::FileOpened)
                }
            }
            Message::FileOpened(result) => {
                self.is_loading = false;
                self.is_dirty = false;
                if let Ok((path, contents)) = result {
                    self.file = Some(path);
                    self.content = text_editor::Content::with_text(&contents);
                }
                Command::none()
            }
            Message::SaveFile => {
                if self.is_loading {
                    Command::none()
                } else {
                    self.is_loading = true;
                    Command::perform(
                        save_file(self.file.clone(), self.content.text()),
                        Message::FileSaved,
                    )
                }
            }
            Message::FileSaved(result) => {
                self.is_loading = false;
                if let Ok(path) = result {
                    self.file = Some(path);
                    self.is_dirty = false;
                }
                Command::none()
            }
            Message::ExitApp => window::close(window::Id::MAIN),
            Message::ZoomIn => {
                self.scale *= 1.1;
                Command::none()
            }
            Message::ZoomOut => {
                self.scale *= 0.9;
                Command::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        keyboard::on_key_press(|key, modifiers| match key.as_ref() {
            keyboard::Key::Character("n") if modifiers.command() => Some(Message::NewFile),
            keyboard::Key::Character("o") if modifiers.command() => Some(Message::OpenFile),
            keyboard::Key::Character("s") if modifiers.command() => Some(Message::SaveFile),
            keyboard::Key::Character("w") if modifiers.command() => Some(Message::ExitApp),
            keyboard::Key::Character("+") if modifiers.command() => Some(Message::ZoomIn),
            keyboard::Key::Character("-") if modifiers.command() => Some(Message::ZoomOut),
            _ => None,
        })
    }

    fn view(&self) -> Element<Message> {
        let controls = row![
            action(new_icon(), "New file", Some(Message::NewFile)),
            action(
                open_icon(),
                "Open file",
                (!self.is_loading).then_some(Message::OpenFile)
            ),
            action(
                save_icon(),
                "Save file",
                self.is_dirty.then_some(Message::SaveFile)
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
            .text_size(14)
            .padding([5, 10])
        ]
        .spacing(10)
        .align_items(Alignment::Center);

        let status = row![
            text(if let Some(path) = &self.file {
                let path = path.display().to_string();
                if path.len() > 60 {
                    format!("...{}", &path[path.len() - 40..])
                } else {
                    path
                }
            } else {
                String::from("New file")
            }),
            horizontal_space(),
            text({
                let (line, column) = self.content.cursor_position();
                format!("{}:{}", line + 1, column + 1)
            })
        ]
        .spacing(10);

        column![
            controls,
            text_editor(&self.content)
                .height(Length::Fill)
                .on_action(Message::ActionPerformed)
                .highlight::<Highlighter>(
                    highlighter::Settings {
                        theme: self.highlighter_theme,
                        extension: self
                            .file
                            .as_deref()
                            .and_then(Path::extension)
                            .and_then(ffi::OsStr::to_str)
                            .map(str::to_string)
                            .unwrap_or(String::from("rs")),
                    },
                    |highlight, _theme| highlight.to_format()
                ),
            status,
        ]
        .spacing(10)
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

#[derive(Debug, Clone)]
pub enum Error {
    DialogClosed,
    IoError(io::ErrorKind),
}

async fn open_file() -> Result<(PathBuf, Arc<String>), Error> {
    let picked_file = rfd::AsyncFileDialog::new()
        .set_title("Open a text file...")
        .pick_file()
        .await
        .ok_or(Error::DialogClosed)?;

    load_file(picked_file).await
}

async fn load_file(path: impl Into<PathBuf>) -> Result<(PathBuf, Arc<String>), Error> {
    let path = path.into();
    let contents = tokio::fs::read_to_string(&path)
        .await
        .map(Arc::new)
        .map_err(|error| Error::IoError(error.kind()))?;

    Ok((path, contents))
}

async fn save_file(path: Option<PathBuf>, contents: String) -> Result<PathBuf, Error> {
    let path = if let Some(path) = path {
        path
    } else {
        rfd::AsyncFileDialog::new()
            .save_file()
            .await
            .as_ref()
            .map(rfd::FileHandle::path)
            .map(Path::to_owned)
            .ok_or(Error::DialogClosed)?
    };

    tokio::fs::write(&path, contents)
        .await
        .map_err(|error| Error::IoError(error.kind()))?;

    Ok(path)
}

fn action<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    label: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let action = button(container(content).center_x(30));
    if let Some(on_press) = on_press {
        tooltip(
            action.on_press(on_press),
            label,
            tooltip::Position::FollowCursor,
        )
        .style(container::rounded_box)
        .into()
    } else {
        action.style(button::secondary).into()
    }
}

fn new_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e800}')
}

fn save_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e801}')
}

fn open_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0f115}')
}

fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("editor-icons");
    text(codepoint).font(ICON_FONT).into()
}
