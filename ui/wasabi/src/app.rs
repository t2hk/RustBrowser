use crate::alloc::string::ToString;
use alloc::format;
use alloc::rc::Rc;
use core::cell::RefCell;
use noli::error::Result as OsResult;
use noli::prelude::SystemApi;
use noli::println;
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
}

impl WasabiUI {
    /// WasabiUI 構造体のコンストラクタ
    pub fn new(browser: Rc<RefCell<Browser>>) -> Self {
        Self {
            browser,
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
        // マウスの位置を取得する。
        loop {
            self.handle_mouse_input()?;
        }
    }

    /// マウスの位置を取得する。
    /// OS が提供する noli ライブラリの Api::get_mouse_cursor_info 関数を使用する。
    /// これは戻り値で マウスクリックの状態とマウスの位置を保持する MouseEvent 構造体を返す。
    fn handle_mouse_input(&mut self) -> Result<(), Error> {
        if let Some(MouseEvent {
            button: _button,
            position,
        }) = Api::get_mouse_cursor_info()
        {
            println!("mouse position {:?}", position);
        }
        Ok(())
    }
}
