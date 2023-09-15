use std::{fs, time::Duration};

use crossterm::event::{self, poll, Event, KeyCode, KeyEvent, KeyModifiers};
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};
use ratatui::{
    style::Style,
    text::Line,
    widgets::{Block, Borders, Paragraph},
};
use tui_input::backend::crossterm::EventHandler;

mod chunks;
mod folder;

mod terminal;

#[derive(Clone)]
enum InputEvent {
    Folder,
    Touch,
}

fn main() -> anyhow::Result<()> {
    if let Some(mut home) = home::home_dir() {
        home.push(".rexplorer/log");

        fs::create_dir_all(home.clone())?;

        home.push("output.log");

        fs::write(home.clone(), "")?;

        // Set up logging
        let logfile = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
            .build(home)?;

        let config = Config::builder()
            .appender(Appender::builder().build("logfile", Box::new(logfile)))
            .build(
                Root::builder()
                    .appender("logfile")
                    .build(log::LevelFilter::Trace),
            )?;

        log4rs::init_config(config)?;
    }

    let mut term = terminal::setup_terminal()?;
    let run_result = run(&mut term);

    terminal::restore_terminal(term)?;

    run_result
}

fn run(terminal: &mut terminal::Terminal) -> anyhow::Result<()> {
    let mut folder = folder::Folder::from("./");

    let mut input = tui_input::Input::default();
    let mut input_active: Option<InputEvent> = None;

    let mut controls_active = false;
    let mut attempting_delete = false;

    loop {
        terminal.draw(|frame| {
            let chunks = chunks::Chunks::new(frame);
            frame.render_widget(
                Paragraph::new(folder.name()).block(Block::new().borders(Borders::ALL)),
                chunks.title(),
            );

            match controls_active {
                true => {
                    frame.render_widget(
                        Paragraph::new(vec![
                            Line::from(" <q> QUIT "),
                            Line::from(" <f> NEW FOLDER "),
                            Line::from(" <t> TOUCH FILE "),
                            Line::from(" <← → ↑ ↓> NAVIGATE "),
                            Line::from(" <CTRL + DEL> DELETE "),
                        ])
                        .block(
                            Block::new()
                                .title("Controls")
                                .borders(Borders::ALL)
                                .style(Style::new().fg(ratatui::style::Color::LightCyan)),
                        ),
                        chunks.main(),
                    );
                }
                false => {
                    frame.render_widget(
                        folder
                            .as_list(&chunks)
                            .block(Block::new().title("Items").borders(Borders::ALL)),
                        chunks.main(),
                    );
                }
            }

            if attempting_delete {
                frame.render_widget(
                    Paragraph::new("Are You Sure? (Y/N)").block(
                        Block::new()
                            .title("Confirm")
                            .borders(Borders::ALL)
                            .border_type(ratatui::widgets::BorderType::Rounded),
                    ),
                    chunks.input_popup(),
                );
            }

            if input_active.is_some() {
                frame.render_widget(
                    Paragraph::new(input.value()).block(
                        Block::new()
                            .title("Input")
                            .borders(Borders::ALL)
                            .border_type(ratatui::widgets::BorderType::Rounded),
                    ),
                    chunks.input_popup(),
                );
            }
        })?;

        if poll(Duration::from_millis(500))? {
            let event = event::read()?;

            if let event::Event::Key(key_event) = event {
                if attempting_delete {
                    match key_event {
                        KeyEvent {
                            code: KeyCode::Char('y'),
                            ..
                        } => {
                            match folder.delete() {
                                Ok(_) => {}
                                Err(_) => log::error!("Item could not be deleted"),
                            }
                            folder = folder.reload();
                            attempting_delete = false;
                        }
                        _ => {
                            attempting_delete = false;
                        }
                    }
                }

                if let Some(input_type) = input_active.clone() {
                    match key_event {
                        KeyEvent {
                            code: KeyCode::Esc, ..
                        } => {
                            input_active = None;
                            input = input.with_value("".to_string());
                        }
                        KeyEvent {
                            code: KeyCode::Enter,
                            ..
                        } => {
                            match input_type {
                                InputEvent::Folder => {
                                    input_active = None;
                                    let value = input.value().to_string();

                                    if value != *"" {
                                        folder.make_folder(value).unwrap();
                                    }

                                    folder = folder.reload();
                                }
                                InputEvent::Touch => {
                                    input_active = None;
                                    let value = input.value().to_string();

                                    if value != *"" {
                                        folder
                                            .touch_file(value)
                                            .expect("File should be able to be created");
                                    }

                                    folder = folder.reload();
                                }
                            }
                            input = input.with_value("".to_string());
                        }
                        _ => {
                            input.handle_event(&Event::Key(key_event));
                        }
                    }
                } else {
                    match key_event {
                        KeyEvent {
                            code: KeyCode::Char('q'),
                            ..
                        } => {
                            break;
                        }
                        KeyEvent {
                            code: KeyCode::Char('f'),
                            ..
                        } => {
                            input_active = Some(InputEvent::Folder);
                            controls_active = false;
                        }
                        KeyEvent {
                            code: KeyCode::Char('t'),
                            ..
                        } => {
                            input_active = Some(InputEvent::Touch);
                            controls_active = false
                        }
                        KeyEvent {
                            code: KeyCode::Char('d'),
                            modifiers: KeyModifiers::CONTROL,
                            ..
                        } => {
                            attempting_delete = true;
                        }
                        KeyEvent {
                            code: KeyCode::Char(' '),
                            ..
                        } => {
                            controls_active = !controls_active;
                        }
                        KeyEvent {
                            code: KeyCode::Up, ..
                        } => folder.scroll_up(),
                        KeyEvent {
                            code: KeyCode::Down,
                            ..
                        } => folder.scroll_down(),
                        KeyEvent {
                            code: KeyCode::Right,
                            ..
                        } => {
                            if let Some(folder_entered) = folder.enter() {
                                folder = folder_entered;
                            } else {
                                log::trace!("{} is not a folder", folder.file_name());
                            }
                        }
                        KeyEvent {
                            code: KeyCode::Left,
                            ..
                        } => {
                            if let Some(back_one) = folder.exit() {
                                folder = back_one
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}
