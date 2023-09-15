use ratatui::prelude::*;

use crate::terminal::Frame;

pub struct Chunks {
    title: Rect,
    main: Rect,
    input_popup: Rect,
}

impl Chunks {
    pub fn new(frame: &mut Frame) -> Self {
        let chunks = Layout::new()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(8)])
            .split(frame.size());

        let simple_split = Layout::new()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Length(3),
                Constraint::Percentage(50),
            ])
            .split(frame.size());

        let input_popup = Layout::new()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
            ])
            .split(simple_split[1])[1];

        Self {
            title: chunks[0],
            main: chunks[1],
            input_popup,
        }
    }
    pub fn title(&self) -> Rect {
        self.title
    }

    pub fn main(&self) -> Rect {
        self.main
    }

    pub fn input_popup(&self) -> Rect {
        self.input_popup
    }
}
