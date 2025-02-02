use crate::alloc::string::ToString;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use core::cell::RefCell;
use noli::error::Result as OsResult;
use noli::prelude::SystemApi;
use noli::println;
use noli::rect::Rect;
use noli::sys::api::MouseEvent;
use noli::sys::wasabi::Api;
use noli::window::StringSize;
use noli::window::Window;
use saba_core::browser::Browser;
use saba_core::constants::WHITE;
use saba_core::constants::WINDOW_HEIGHT;
use saba_core::constants::WINDOW_INIT_X_POS;
use saba_core::constants::WINDOW_INIT_Y_POS;
use saba_core::constants::WINDOW_WIDTH;
use saba_core::constants::*;
use saba_core::error::Error;

/// WasabiUI 構造体
/// ウィンドウのインスタンスとブラウザの実装を保持する。
#[derive(Debug)]
pub struct WasabiUI {
    // ブラウザは Rc/RefCell により、複数の個所から参照できるようにする。
    browser: Rc<RefCell<Browser>>,
    window: Window,
    input_mode: InputMode,
    input_url: String,
}

/// InputMode 列挙型
/// 現在のアプリケーションが文字を入力できる状態かどうかを表す。
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum InputMode {
    Normal,  // 文字を入力できない状態
    Editing, // 文字を入力できる状態
}

impl WasabiUI {
    /// WasabiUI 構造体のコンストラクタ
    pub fn new(browser: Rc<RefCell<Browser>>) -> Self {
        Self {
            browser,
            input_url: String::new(),
            input_mode: InputMode::Normal,
            window: Window::new(
                "saba".to_string(),
                WHITE,
                WINDOW_INIT_X_POS,
                WINDOW_INIT_Y_POS,
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
            )
            .unwrap(),
        }
    }

    /// ツールバーの描画
    fn setup_toolbar(&mut self) -> OsResult<()> {
        // ツールバーの背景の四角を描画
        self.window
            .fill_rect(LIGHTGREY, 0, 0, WINDOW_WIDTH, TOOLBAR_HEIGHT)?;

        // ツールバーとコンテンツエリアの境目の線を描画
        self.window
            .draw_line(GREY, 0, TOOLBAR_HEIGHT, WINDOW_WIDTH - 1, TOOLBAR_HEIGHT)?;
        self.window.draw_line(
            DARKGREY,
            0,
            TOOLBAR_HEIGHT + 1,
            WINDOW_WIDTH - 1,
            TOOLBAR_HEIGHT + 1,
        )?;

        // アドレスバーの横に "Address" という文字列を描画
        self.window.draw_string(
            BLACK,
            5,
            5,
            "Address:",
            StringSize::Medium,
            /*underline=*/ false,
        )?;

        // アドレスバーの四角を描画
        self.window
            .fill_rect(WHITE, 70, 2, WINDOW_WIDTH - 74, 2 + ADDRESSBAR_HEIGHT)?;

        // アドレスバーの影の線を描画
        self.window.draw_line(GREY, 70, 2, WINDOW_WIDTH - 4, 2)?;
        self.window
            .draw_line(GREY, 70, 2, 70, 2 + ADDRESSBAR_HEIGHT)?;
        self.window.draw_line(BLACK, 71, 3, WINDOW_WIDTH - 5, 3)?;
        self.window
            .draw_line(GREY, 71, 3, 71, 1 + ADDRESSBAR_HEIGHT)?;

        Ok(())
    }

    /// ウィンドウの初期化
    fn setup(&mut self) -> Result<(), Error> {
        if let Err(error) = self.setup_toolbar() {
            // OsResult と Result が持つ Error 型は異なるので、変換する。
            return Err(Error::InvalidUI(format!(
                "failded to initialized a toolbar with error: {:#?}",
                error
            )));
        }
        // 画面を更新する。
        self.window.flush();
        Ok(())
    }

    /// UI を開始するメソッド
    pub fn start(&mut self) -> Result<(), Error> {
        self.setup()?;
        self.run_app()?;

        Ok(())
    }

    /// アプリケーションを実行するための関数
    fn run_app(&mut self) -> Result<(), Error> {
        loop {
            // マウスの位置を取得する。
            self.handle_mouse_input()?;
            // キー入力を取得する。
            self.handle_key_input()?;
        }
    }

    /// マウスの位置を取得する。
    /// OS が提供する noli ライブラリの Api::get_mouse_cursor_info 関数を使用する。
    /// これは戻り値で マウスクリックの状態とマウスの位置を保持する MouseEvent 構造体を返す。
    fn handle_mouse_input(&mut self) -> Result<(), Error> {
        if let Some(MouseEvent { button, position }) = Api::get_mouse_cursor_info() {
            if button.l() || button.c() || button.r() {
                // 相対位置を計算する。
                let relative_pos = (
                    position.x - WINDOW_INIT_X_POS,
                    position.y - WINDOW_INIT_Y_POS,
                );

                // ウィンドウの外をクリックされたときは何もしない。
                if relative_pos.0 < 0
                    || relative_pos.0 > WINDOW_WIDTH
                    || relative_pos.1 < 0
                    || relative_pos.1 > WINDOW_HEIGHT
                {
                    println!("button clicked OUTSIDE window: {button:?} {position:?}");
                    return Ok(());
                }

                // ツールバーの範囲をクリックされたとき、InputMode を Editing に変更する。
                if relative_pos.1 < TOOLBAR_HEIGHT + TITLE_BAR_HEIGHT
                    && relative_pos.1 >= TITLE_BAR_HEIGHT
                {
                    self.clear_address_bar()?;
                    self.input_url = String::new();
                    self.input_mode = InputMode::Editing;
                    println!("button clicked in toolbar: {button:?} {position:?}");
                    return Ok(());
                }
                self.input_mode = InputMode::Normal;
            }
            // println!("mouse position {:?}", position);
            // if button.l() || button.c() || button.r() {
            //     println!("mouse clicked {:?}", button);
            // }
        }
        Ok(())
    }

    /// 文字を入力する。
    /// noli の Api::read_key 関数は文字入力に対して1文字を返す。
    fn handle_key_input(&mut self) -> Result<(), Error> {
        match self.input_mode {
            InputMode::Normal => {
                // InputMode が Normal の時、キー入力を無視する。
                let _ = Api::read_key();
            }
            InputMode::Editing => {
                if let Some(c) = Api::read_key() {
                    if c == 0x7F as char || c == 0x08 as char {
                        // デリートキーまたはバックスペースキーが押された場合、最後の文字を削除する。
                        // input_url の文字列を変更した後は update_address_bar を呼んで描画する。
                        self.input_url.pop();
                        self.update_address_bar()?;
                    } else {
                        self.input_url.push(c);
                        self.update_address_bar()?;
                    }
                }
            }
        }
        // if let Some(c) = Api::read_key() {
        //     println!("input text: {:?}", c);
        // }
        Ok(())
    }

    /// URL の情報をツールバーに反映する。
    /// fill_rect や draw_string などの描画 API は呼び出した時点で描画せず、flush_area を呼び出したタイミングで描画される。
    fn update_address_bar(&mut self) -> Result<(), Error> {
        // アドレスバーを白く塗りつぶす
        if self
            .window
            .fill_rect(WHITE, 72, 4, WINDOW_WIDTH - 76, ADDRESSBAR_HEIGHT - 2)
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to clear an address bar".to_string(),
            ));
        }

        // input_url をアドレスバーに描画する。
        if self
            .window
            .draw_string(
                BLACK,
                74,
                6,
                &self.input_url,
                StringSize::Medium,
                /*underline=*/ false,
            )
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to update an address bar".to_string(),
            ));
        }

        self.window.flush_area(
            Rect::new(
                WINDOW_INIT_X_POS,
                WINDOW_INIT_Y_POS + TITLE_BAR_HEIGHT,
                WINDOW_WIDTH,
                TOOLBAR_HEIGHT,
            )
            .expect("failed to create a rect for the address bar"),
        );

        Ok(())
    }

    /// アドレスバーに入力されている URL を消去する。
    fn clear_address_bar(&mut self) -> Result<(), Error> {
        // アドレスバーを白く塗りつぶす。
        if self
            .window
            .fill_rect(WHITE, 72, 4, WINDOW_WIDTH - 76, ADDRESSBAR_HEIGHT - 2)
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to clear an address bar".to_string(),
            ));
        }

        // アドレスバーの部分の描画を更新する。
        self.window.flush_area(
            Rect::new(
                WINDOW_INIT_X_POS,
                WINDOW_INIT_Y_POS + TITLE_BAR_HEIGHT,
                WINDOW_WIDTH,
                TOOLBAR_HEIGHT,
            )
            .expect("failed to create a rect for the address bar"),
        );

        Ok(())
    }
}
