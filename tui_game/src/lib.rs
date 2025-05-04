use std::{
    fmt::Debug,
    str::{FromStr, SplitWhitespace},
    sync::Arc,
};

use anyhow::Context;
use async_trait::async_trait;
use board_game::{
    game_executor::{GameExecutor, GameExecutorError},
    message::{
        ExecutorToManagerMsg, ManagerToExecutorMsg, request_message::ExecutorToManagerReqMsg,
    },
    player::Player,
};

use color_eyre::owo_colors::OwoColorize;
use crossterm::event::EventStream;
use futures::{FutureExt, StreamExt};
use ratatui::{
    Frame, Terminal,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::Backend,
    style::{Color, Modifier, Style},
    text::{Span, Text},
    widgets::{Block, Borders, Paragraph},
};
use tokio::{
    sync::{Mutex, Notify, mpsc},
    task::spawn_blocking,
};
use tracing::{debug, info, trace};
use tracing_subscriber::field::debug;
use ui::{
    screen::{
        game_on_screen::GameOnScreen,
        main_screen::{self, CurrentSelectMenu, MainScreen, SELECT_MENU_NUMS},
    },
    state::CurrentScreen,
};

mod ui;

pub struct TuiGameExecutor<B: Backend + std::marker::Send + std::marker::Sync> {
    tx: Option<Arc<Mutex<mpsc::Sender<ExecutorToManagerMsg>>>>,
    rx: Option<Arc<Mutex<mpsc::Receiver<ManagerToExecutorMsg>>>>,
    is_win: Arc<Mutex<bool>>,
    is_win_notify: Arc<Notify>,
    is_tile_on_notify: Arc<Notify>,
    terminal: Arc<Mutex<Terminal<B>>>,
    current_screen: Arc<Mutex<CurrentScreen>>,
    quit_game_now: Arc<Mutex<bool>>,
    event_strem: Arc<Mutex<EventStream>>,
}

impl<B: Backend + std::marker::Send + std::marker::Sync> TuiGameExecutor<B> {
    pub fn new(terminal: Arc<Mutex<Terminal<B>>>) -> Self {
        Self {
            tx: None,
            rx: None,
            is_win: Arc::new(Mutex::new(false)),
            is_win_notify: Arc::new(Notify::new()),
            is_tile_on_notify: Arc::new(Notify::new()),
            terminal,
            current_screen: Arc::new(Mutex::new(CurrentScreen::Main(MainScreen::new()))),
            quit_game_now: Arc::new(Mutex::new(false)),
            event_strem: Arc::new(Mutex::new(EventStream::new())),
        }
    }

    pub fn set_rx(mut self, rx: mpsc::Receiver<ManagerToExecutorMsg>) -> Self {
        self.rx = Some(Arc::new(Mutex::new(rx)));
        self
    }

    pub fn set_tx(mut self, tx: mpsc::Sender<ExecutorToManagerMsg>) -> Self {
        self.tx = Some(Arc::new(Mutex::new(tx)));
        self
    }

    pub fn ui(current_screen: &mut CurrentScreen, frame: &mut Frame<'_>) {
        if let CurrentScreen::Main(main_screen) = current_screen {
            let title_block = Block::default()
                .borders(Borders::ALL)
                .style(Style::default());

            let title_text = Text::from(Span::styled(
                "Tic-Tac-Toe",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Green),
            ));

            let title_paragraph_block = Paragraph::new("").block(title_block).centered();
            let title_area = Self::centered_rect_at(20, 10, 60, 25, frame.area());
            let title_paragraph_text = Paragraph::new(title_text).centered();
            frame.render_widget(title_paragraph_block, title_area);
            frame.render_widget(
                title_paragraph_text,
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Fill(1),
                        Constraint::Length(1),
                        Constraint::Fill(1),
                    ])
                    .split(title_area)[1],
            );

            let menu_chuncks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Fill(1),
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Fill(1),
                    ]
                    .as_ref(),
                )
                .split(Self::centered_rect_at(20, 50, 60, 35, frame.area()));

            let mut start_block = Block::default().borders(Borders::ALL);
            let mut quit_block = Block::default().borders(Borders::ALL);
            let mut start_text = Text::styled("Start Game", Style::default());
            let mut quit_text = Text::styled("Quit Game", Style::default());
            match main_screen.menu_select {
                Some(CurrentSelectMenu::StartGame) => {
                    start_text = start_text.style(
                        Style::default()
                            .fg(Color::LightRed)
                            .add_modifier(Modifier::BOLD),
                    );
                    start_block = start_block.style(Style::default().bg(Color::Gray));
                }
                Some(CurrentSelectMenu::QuitGame) => {
                    quit_text = quit_text.style(
                        Style::default()
                            .fg(Color::LightRed)
                            .add_modifier(Modifier::BOLD),
                    );
                    quit_block = quit_block.style(Style::default().bg(Color::Gray));
                }
                None => {}
            }
            let start_paragraph = Paragraph::new(start_text).block(start_block).centered();
            let quit_paragraph = Paragraph::new(quit_text).block(quit_block).centered();
            frame.render_widget(start_paragraph, menu_chuncks[1]);
            frame.render_widget(quit_paragraph, menu_chuncks[2]);
        }
    }

    async fn analyze_input(&self) -> Result<(), GameExecutorError> {
        debug!("start analyzing input");
        match *self.current_screen.lock().await {
            CurrentScreen::Main(ref mut main_screen) => {
                debug!("match successfully");
                match self.event_strem.lock().await.next().fuse().await {
                    Some(Ok(Event::Key(key_event))) => {
                        if key_event.kind == KeyEventKind::Press {
                            match key_event.code {
                                KeyCode::Char('q') | KeyCode::Esc => {
                                    main_screen.menu_select = Some(CurrentSelectMenu::QuitGame);
                                }
                                KeyCode::Up => {
                                    if let Some(ref mut current_select) = main_screen.menu_select {
                                        if *current_select as usize != 0 {
                                            *current_select = CurrentSelectMenu::try_from(
                                                *current_select as usize - 1,
                                            )
                                            .unwrap();
                                        }
                                    } else {
                                        main_screen.menu_select =
                                            Some(CurrentSelectMenu::StartGame);
                                    }
                                }
                                KeyCode::Down => {
                                    if let Some(ref mut current_select) = main_screen.menu_select {
                                        if *current_select as usize != SELECT_MENU_NUMS - 1 {
                                            *current_select = CurrentSelectMenu::try_from(
                                                *current_select as usize + 1,
                                            )
                                            .unwrap();
                                        }
                                    } else {
                                        main_screen.menu_select =
                                            Some(CurrentSelectMenu::try_from(0).unwrap());
                                    }
                                }
                                KeyCode::Tab => {
                                    if let Some(ref mut current_select) = main_screen.menu_select {
                                        if *current_select as usize != SELECT_MENU_NUMS - 1 {
                                            *current_select = CurrentSelectMenu::try_from(
                                                *current_select as usize + 1,
                                            )
                                            .unwrap();
                                        } else {
                                            *current_select =
                                                CurrentSelectMenu::try_from(0).unwrap();
                                        }
                                    } else {
                                        main_screen.menu_select =
                                            Some(CurrentSelectMenu::try_from(0).unwrap());
                                    }
                                }
                                KeyCode::Enter => {
                                    if let Some(current_select) = main_screen.menu_select {
                                        match current_select {
                                            CurrentSelectMenu::StartGame => {
                                                *self.current_screen.lock().await =
                                                    CurrentScreen::GameOn(GameOnScreen::new());
                                            }
                                            CurrentSelectMenu::QuitGame => {
                                                *self.quit_game_now.lock().await = true;
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(err)) => {
                        return Err(GameExecutorError::from(err));
                    }
                    None => {}
                };
                Ok(())
            }
            CurrentScreen::GameOn(ref mut game_on_screen) => {
                match self.event_strem.lock().await.next().fuse().await {
                    Some(Ok(Event::Key(key_event))) => {
                        if let KeyEventKind::Press = key_event.kind {
                            match key_event.code {
                                KeyCode::Up => {}
                                _ => {}
                            }
                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(err)) => {
                        return Err(GameExecutorError::from(err));
                    }
                    None => (),
                }
                Ok(())
            }
        }
    }

    fn centered_rect_at(
        start_x: u16,
        start_y: u16,
        percent_x: u16,
        percent_y: u16,
        rect: Rect,
    ) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(start_y),
                    Constraint::Percentage(percent_y),
                    Constraint::Percentage(100 - percent_y - start_y),
                ]
                .as_ref(),
            )
            .split(rect);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(start_x),
                    Constraint::Percentage(percent_x),
                    Constraint::Percentage(100 - percent_x - start_x),
                ]
                .as_ref(),
            )
            .split(popup_layout[1])[1]
    }

    fn centered_rect(percent_x: u16, percent_y: u16, rect: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_y) / 2),
                    Constraint::Percentage(percent_y),
                    Constraint::Percentage((100 - percent_y) / 2),
                ]
                .as_ref(),
            )
            .split(rect);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_x) / 2),
                    Constraint::Percentage(percent_x),
                    Constraint::Percentage((100 - percent_x) / 2),
                ]
                .as_ref(),
            )
            .split(popup_layout[1])[1]
    }
}

// 新增函数：使用 ratatui 读取两个整数 posx 和 posy
async fn read_pos_from_tui<B: Backend>(
    terminal: Arc<Mutex<Terminal<B>>>,
) -> Result<(usize, usize), GameExecutorError> {
    let mut posx = String::new();
    let mut posy = String::new();
    let mut focus_on_posx = true;

    loop {
        terminal
            .lock()
            .await
            .draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Length(3)].as_ref())
                    .split(f.size());

                let input_style = Style::default().fg(Color::White).bg(Color::Black);
                let focused_style = Style::default().fg(Color::Black).bg(Color::White);

                let posx_input =
                    Paragraph::new(format!("posx: {}", posx)).style(if focus_on_posx {
                        focused_style
                    } else {
                        input_style
                    });
                let posy_input =
                    Paragraph::new(format!("posy: {}", posy)).style(if !focus_on_posx {
                        focused_style
                    } else {
                        input_style
                    });

                f.render_widget(posx_input, chunks[0]);
                f.render_widget(posy_input, chunks[1]);
            })
            .context("Failed to draw UI")?;

        if let Event::Key(KeyEvent {
            code,
            modifiers,
            kind,
            state,
        }) = event::read().context("Failed to read event from terminal")?
        {
            match (code, modifiers) {
                (KeyCode::Char(c), _) if focus_on_posx => posx.push(c),
                (KeyCode::Char(c), _) if !focus_on_posx => posy.push(c),
                (KeyCode::Backspace, _) if focus_on_posx && !posx.is_empty() => {
                    posx.pop();
                }
                (KeyCode::Backspace, _) if !focus_on_posx && !posy.is_empty() => {
                    posy.pop();
                }
                (KeyCode::Tab, _) => focus_on_posx = !focus_on_posx,
                (KeyCode::Enter, _) => {
                    let posx_val = posx.parse::<usize>().unwrap_or(0);
                    let posy_val = posy.parse::<usize>().unwrap_or(0);
                    return Ok((posx_val, posy_val));
                }
                _ => {}
            }
        }
    }
}

#[async_trait]
impl<B: Backend + std::marker::Sync + std::marker::Send> GameExecutor for TuiGameExecutor<B> {
    fn get_tx(&self) -> Result<Arc<Mutex<mpsc::Sender<ExecutorToManagerMsg>>>, GameExecutorError> {
        Ok(self.tx.clone().unwrap())
    }

    fn get_rx(
        &self,
    ) -> Result<Arc<Mutex<mpsc::Receiver<ManagerToExecutorMsg>>>, GameExecutorError> {
        Ok(self.rx.clone().unwrap())
    }

    async fn init_game_impl(&self) -> Result<(), GameExecutorError> {
        trace!("init game");
        Ok(())
    }

    async fn quit_game_impl(&self) -> Result<(), GameExecutorError> {
        trace!("execute quiting game");
        Ok(())
    }

    async fn execute_game_impl(&self) -> Result<(), GameExecutorError> {
        let tx = self.get_tx()?;
        trace!("start executing game");
        loop {
            let mut current_screen = self.current_screen.lock().await;
            self.terminal.lock().await.draw(|frame| {
                Self::ui(&mut current_screen, frame);
            })?;
            debug!("drawing");
            drop(current_screen);
            self.analyze_input().await?;
            debug!("finalize input");
            if *self.quit_game_now.lock().await {
                break;
            }
            debug!("complete one loop");
        }
        tx.lock()
            .await
            .send(ExecutorToManagerMsg::Request(
                ExecutorToManagerReqMsg::ReadyToQuitGameRequest,
            ))
            .await?;
        Ok(())
    }

    async fn player_win_impl(&self, player: &Player) -> Result<(), GameExecutorError> {
        self.is_win_notify.notify_one();
        info!("player {} Win", player.id);
        Ok(())
    }

    async fn process_tile_on_by_player_response(
        &self,
        is_win: bool,
    ) -> Result<(), GameExecutorError> {
        trace!("process tile on by player response");
        *self.is_win.lock().await = is_win;
        self.is_tile_on_notify.notify_one();
        Ok(())
    }
}
// struct Reader<'a> {
//     buf: *mut String,
//     nums: Option<SplitWhitespace<'a>>,
// }

// impl Reader<'_> {
//     fn new() -> Self {
//         Self {
//             buf: std::ptr::null_mut(),
//             nums: None,
//         }
//     }

//     fn get_line(&mut self) {
//         let mut new_line = Box::new(String::new());
//         std::io::stdin().read_line(&mut new_line).unwrap();
//         if !self.buf.is_null() {
//             let _ = unsafe { Box::from_raw(self.buf) };
//         }
//         self.buf = Box::into_raw(new_line);
//         self.nums = unsafe { Some((*self.buf).split_whitespace()) };
//     }

//     fn read_token(&mut self) -> &str {
//         if self.nums.is_none() {
//             self.get_line();
//         }

//         loop {
//             match self.nums.as_mut().unwrap().next() {
//                 Some(val) => {
//                     break val;
//                 }
//                 None => {
//                     self.get_line();
//                 }
//             }
//         }
//     }

//     fn read_int<T: FromStr>(&mut self) -> T
//     where
//         <T as FromStr>::Err: Debug,
//     {
//         self.read_token().parse().unwrap()
//     }
// }

// impl Drop for Reader<'_> {
//     fn drop(&mut self) {
//         if !self.buf.is_null() {
//             let _ = unsafe { Box::from_raw(self.buf) };
//         }
//     }
// }

// unsafe impl Send for Reader<'_> {}
