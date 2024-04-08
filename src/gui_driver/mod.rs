

use ::iced::theme::Theme;
use iced::widget::Text;
use iced::widget::button::Button;
use iced::{
    Alignment, Sandbox, Settings, Element
};


pub fn gui_runtime() -> iced::Result { 
    RustUI::run(Settings::default())
}


struct RustUI {
    theme: Theme,
    page: Page
}

enum Page {
    Monitoring,
    Controling
}

#[derive(Debug, Clone)]
enum Message { 
    ToggleTheme,    // light/ dark theme
    Router(String), //change page
}

impl Sandbox for RustUI {
    type Message = Message;

    fn new() -> Self {
        Self {
            theme: Theme::Light,
            page: Page::Monitoring
        }
    }

    fn title(&self) -> String {
        String::from("RustUI")
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ToggleTheme => {}
            Message::Router(route) => {}
        }
    }

    fn view(&self) -> Element<Message> {
        Button::new(Text::new("Toggle Theme"))
            .on_press(Message::ToggleTheme)
            .into()
    }
}

