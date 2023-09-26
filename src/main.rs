use std::{cell::RefMut, error::Error, time::Duration};

use folder::{Folder, FolderChunk};
use tui_input::backend::crossterm::EventHandler;
use widgetui::{
    crossterm::event::{Event, KeyCode},
    ratatui::{
        prelude::*,
        widgets::{Block, Borders, Paragraph},
    },
    widgets::message::{Message, MessageChunk, MessageState},
    *,
};

mod folder;

#[derive(FromState)]
pub struct TextInput(tui_input::Input);

#[derive(FromState, PartialEq, Clone)]
pub enum AppState {
    Navigation,
    Controls,
    NewFolder,
    TouchFile,
    ConfirmDelete,
}

const BLOCK: Block = Block::new()
    .borders(Borders::ALL)
    .border_type(ratatui::widgets::BorderType::Plain);

pub struct TitleChunk;
pub struct InputChunk;

fn chunk_builder(frame: &mut WidgetFrame, mut chunks: RefMut<Chunks>) -> WidgetResult {
    let layout = layout!(frame.size(), constraint!(#3), constraint!(%100));

    let input_chunk = layout!(
        frame.size(),
        constraint!(%50),
        constraint!(#3) => {constraint!(#3), constraint!(>3), constraint!(#3)},
        constraint!(%50)
    )[1][1];

    chunks.register_chunk::<FolderChunk>(layout[1][0]);
    chunks.register_chunk::<TitleChunk>(layout[0][0]);
    chunks.register_chunk::<InputChunk>(input_chunk);
    chunks.register_chunk::<MessageChunk>(input_chunk);

    Ok(())
}

fn render(
    frame: &mut WidgetFrame,
    chunks: RefMut<Chunks>,
    mut folder: RefMut<Folder>,
    state: RefMut<AppState>,
    input: RefMut<TextInput>,
) -> WidgetResult {
    let folder_chunk = chunks.get_chunk::<FolderChunk>()?;
    if *state == AppState::Controls {
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(" <q> QUIT "),
                Line::from(" <f> NEW FOLDER "),
                Line::from(" <t> TOUCH FILE "),
                Line::from(" <← → ↑ ↓> NAVIGATE "),
                Line::from(" <d> DELETE "),
            ])
            .block(
                BLOCK
                    .title("Controls")
                    .style(Style::new().fg(Color::LightCyan)),
            ),
            folder_chunk,
        )
    } else {
        frame.render_widget(folder.as_list(folder_chunk).block(BLOCK), folder_chunk);
    }

    if *state == AppState::ConfirmDelete {
        frame.render_widget(
            Paragraph::new("Are you sure? Y/N").block(BLOCK),
            chunks.get_chunk::<InputChunk>()?,
        );
    }

    if *state == AppState::TouchFile {
        frame.render_widget(
            Paragraph::new(input.0.value()).block(BLOCK.title("Enter Filename")),
            chunks.get_chunk::<InputChunk>()?,
        );
    }

    if *state == AppState::NewFolder {
        frame.render_widget(
            Paragraph::new(input.0.value()).block(BLOCK.title("Enter Folder Name")),
            chunks.get_chunk::<InputChunk>()?,
        );
    }

    frame.render_widget(
        Paragraph::new(folder.name()).block(BLOCK),
        chunks.get_chunk::<TitleChunk>()?,
    );

    Ok(())
}

fn controls(
    _frame: &mut WidgetFrame,
    mut events: RefMut<Events>,
    mut state: RefMut<AppState>,
) -> WidgetResult {
    if *state != AppState::Controls {
        return Ok(());
    }

    if let Some(Event::Key(event)) = events.event {
        let code = event.code;

        match code {
            KeyCode::Char('q') => events.register_exit(),
            KeyCode::Char('d') => *state = AppState::ConfirmDelete,
            KeyCode::Char('f') => *state = AppState::NewFolder,
            KeyCode::Char('t') => *state = AppState::TouchFile,
            _ => *state = AppState::Navigation,
        }
        events.event = None;
    }

    Ok(())
}

fn navigation(
    _frame: &mut WidgetFrame,
    mut events: RefMut<Events>,
    mut folder: RefMut<Folder>,
    mut state: RefMut<AppState>,
) -> WidgetResult {
    if *state != AppState::Navigation {
        return Ok(());
    }

    if events.consume_key(KeyCode::Up) {
        folder.scroll_up();
    }
    if events.consume_key(KeyCode::Down) {
        folder.scroll_down();
    }
    if events.consume_key(KeyCode::Right) {
        if let Some(new_folder) = folder.enter() {
            *folder = new_folder;
        }
    }
    if events.consume_key(KeyCode::Left) {
        if let Some(new_folder) = folder.exit() {
            *folder = new_folder;
        }
    }

    // State Exit Event
    if events.consume_key(KeyCode::Char(' ')) {
        *state = AppState::Controls;
    }

    Ok(())
}

fn delete(
    _frame: &mut WidgetFrame,
    mut events: RefMut<Events>,
    mut folder: RefMut<Folder>,
    mut state: RefMut<AppState>,
    mut message: RefMut<MessageState>,
) -> WidgetResult {
    if *state != AppState::ConfirmDelete {
        return Ok(());
    }

    if let Some(Event::Key(event)) = events.event {
        let code = event.code;

        if code == KeyCode::Char('y') {
            if folder.delete().is_err() {
                message.render_message(
                    "ERROR: Unable to delete File/Folder",
                    Duration::from_millis(1000),
                )
            } else {
                *folder = folder.reload();
            }
        }

        *state = AppState::Navigation;

        events.event = None;
    }

    Ok(())
}

fn input(
    _frame: &mut WidgetFrame,
    mut events: RefMut<Events>,
    mut folder: RefMut<Folder>,
    mut state: RefMut<AppState>,
    mut message: RefMut<MessageState>,
    mut input: RefMut<TextInput>,
) -> WidgetResult {
    if *state != AppState::NewFolder && *state != AppState::TouchFile {
        return Ok(());
    }

    if let Some(Event::Key(event)) = events.event {
        let code = event.code;

        match code {
            KeyCode::Enter => match *state {
                AppState::TouchFile => {
                    if folder.touch_file(input.0.value().to_string()).is_err() {
                        message.render_message(
                            "ERROR: Unable to create File",
                            Duration::from_millis(1000),
                        )
                    } else {
                        *folder = folder.reload();
                    }

                    input.0.reset();
                    *state = AppState::Navigation
                }
                AppState::NewFolder => {
                    if folder.make_folder(input.0.value().to_string()).is_err() {
                        message.render_message(
                            "ERROR: Unable to create Folder",
                            Duration::from_millis(1000),
                        )
                    } else {
                        *folder = folder.reload();
                    }

                    input.0.reset();
                    *state = AppState::Navigation
                }
                _ => unreachable!(),
            },
            KeyCode::Esc => {
                input.0.reset();
                *state = AppState::Navigation
            }
            _ => {
                input.0.handle_event(&Event::Key(event));
            }
        }

        events.event = None;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    App::new(500)?
        .with_state(AppState::Navigation)
        .with_state(TextInput(tui_input::Input::new("".to_string())))
        .with_state(folder::Folder::from("./"))
        .with_widget(chunk_builder)
        .with_widget(controls)
        .with_widget(navigation)
        .with_widget(delete)
        .with_widget(input)
        .with_set(Message)
        .with_widget(render)
        .handle_panics()
        .run()
}
