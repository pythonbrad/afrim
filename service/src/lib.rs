mod convert;
pub mod frontend;

pub use afrim_config::Config;
use afrim_preprocessor::{utils, Command as EventCmd, Preprocessor};
use afrim_translator::Translator;
use anyhow::{Context, Result};
use enigo::{Enigo, Keyboard};
use frontend::{Command as GUICmd, Frontend};
use rdev::{self, EventType, Key as E_Key};
use std::{rc::Rc, sync::mpsc, thread};

/// Starts the afrim.
pub fn run(
    config: Config,
    mut frontend: impl Frontend + std::marker::Send + 'static,
) -> Result<()> {
    // State.
    let mut is_ctrl_released = true;
    let mut idle = false;
    let mut output = String::default();

    // Configuration of the afrim.
    let memory = utils::build_map(
        config
            .extract_data()
            .iter()
            .map(|(key, value)| vec![key.as_str(), value.as_str()])
            .collect(),
    );
    let (buffer_size, auto_commit, page_size) = config
        .core
        .as_ref()
        .map(|core| {
            (
                core.buffer_size.unwrap_or(32),
                core.auto_commit.unwrap_or(false),
                core.page_size.unwrap_or(10),
            )
        })
        .unwrap_or((32, false, 10));
    let mut keyboard = Enigo::new(&Default::default()).unwrap();
    let mut preprocessor = Preprocessor::new(Rc::new(memory), buffer_size);
    #[cfg(not(feature = "rhai"))]
    let translator = Translator::new(config.extract_translation(), auto_commit);
    #[cfg(feature = "rhai")]
    let mut translator = Translator::new(config.extract_translation(), auto_commit);
    #[cfg(feature = "rhai")]
    config
        .extract_translators()
        .context("Failed to load translators.")?
        .into_iter()
        .for_each(|(name, ast)| translator.register(name, ast));

    // Configuration of the frontend.
    let (frontend_tx1, frontend_rx1) = mpsc::channel();
    let (frontend_tx2, frontend_rx2) = mpsc::channel();

    frontend_tx1.send(GUICmd::PageSize(page_size))?;
    let screen_size = rdev::display_size().unwrap();
    frontend_tx1.send(GUICmd::ScreenSize(screen_size))?;

    let frontend_thread = thread::spawn(move || {
        frontend
            .init(frontend_tx2, frontend_rx1)
            .context("Failure to initialize the frontend.")
            .unwrap();
        frontend
            .listen()
            .context("The frontend raise an unexpected error.")
            .unwrap();
    });

    // Configuration of the event listener.
    let (event_tx, event_rx) = mpsc::channel();
    thread::spawn(move || {
        rdev::listen(move |event| {
            event_tx
                .send(event)
                .unwrap_or_else(|e| eprintln!("Could not send event {:?}", e));
        })
        .expect("Could not listen");
    });

    // We process event.
    for event in event_rx.iter() {
        let prev_input = preprocessor.get_input();

        match event.event_type {
            // Handling of idle state.
            EventType::KeyPress(E_Key::Pause) => {
                idle = true;
                frontend_tx1.send(GUICmd::State(idle))?;
            }
            EventType::KeyRelease(E_Key::Pause) => {
                idle = false;
                frontend_tx1.send(GUICmd::State(idle))?;
            }
            EventType::KeyPress(E_Key::ControlLeft | E_Key::ControlRight) => {
                is_ctrl_released = false;
            }
            EventType::KeyRelease(E_Key::ControlLeft | E_Key::ControlRight) if is_ctrl_released => {
                idle = !idle;
                frontend_tx1.send(GUICmd::State(idle))?;
            }
            EventType::KeyRelease(E_Key::ControlLeft | E_Key::ControlRight) => {
                is_ctrl_released = true;
            }
            _ if idle => (),
            // Handling of special functions.
            // Select next predicate.
            EventType::KeyRelease(E_Key::ShiftRight) if !is_ctrl_released => {
                frontend_tx1.send(GUICmd::SelectNextPredicate)?;
            }
            // Select previous predicate.
            EventType::KeyRelease(E_Key::ShiftLeft) if !is_ctrl_released => {
                frontend_tx1.send(GUICmd::SelectPreviousPredicate)?;
            }
            // Commit the selected predicate.
            EventType::KeyRelease(E_Key::Space) if !is_ctrl_released => {
                rdev::simulate(&EventType::KeyRelease(E_Key::ControlLeft))
                    .expect("We couldn't cancel the special function key");

                frontend_tx1.send(GUICmd::SelectedPredicate)?;
                if let GUICmd::Predicate(predicate) = frontend_rx2.recv()? {
                    rdev::simulate(&EventType::KeyPress(E_Key::Pause)).unwrap();
                    keyboard
                        .text(predicate.texts.first().unwrap_or(&String::default()))
                        .unwrap();
                    rdev::simulate(&EventType::KeyRelease(E_Key::Pause)).unwrap();

                    preprocessor.process(Default::default());
                    output.clear();
                    frontend_tx1.send(GUICmd::Clear)?;
                }
            }
            _ if !is_ctrl_released => (),
            // Commit the output.
            EventType::KeyPress(E_Key::Space) if !prev_input.is_empty() => {
                rdev::simulate(&EventType::KeyRelease(E_Key::Space)).unwrap();
                rdev::simulate(&EventType::KeyPress(E_Key::Pause)).unwrap();

                // Delete the space.
                rdev::simulate(&EventType::KeyPress(E_Key::Backspace)).unwrap();
                rdev::simulate(&EventType::KeyRelease(E_Key::Backspace)).unwrap();

                // Commit
                keyboard.text(&output).unwrap();

                rdev::simulate(&EventType::KeyRelease(E_Key::Pause)).unwrap();

                preprocessor.process(Default::default());
                output.clear();
                frontend_tx1.send(GUICmd::Clear)?;
            }
            // GUI events.
            EventType::MouseMove { x, y } => {
                frontend_tx1.send(GUICmd::Position((x, y)))?;
            }
            // Process events.
            _ => {
                let (changed, _committed) = preprocessor.process(convert::from_event(&event));
                let curr_input = preprocessor.get_input();

                if prev_input.len() < curr_input.len() {
                    output.push(curr_input.chars().last().unwrap_or_default());

                    // Cancel the keyboard input.
                    rdev::simulate(&EventType::KeyPress(E_Key::Pause)).unwrap();
                    rdev::simulate(&EventType::KeyPress(E_Key::Backspace)).unwrap();
                    rdev::simulate(&EventType::KeyRelease(E_Key::Backspace)).unwrap();
                    rdev::simulate(&EventType::KeyRelease(E_Key::Pause)).unwrap();
                } else if curr_input.is_empty() {
                    output.clear();
                }

                if changed {
                    frontend_tx1.send(GUICmd::Clear)?;

                    translator
                        .translate(&curr_input)
                        .into_iter()
                        .take(page_size * 2)
                        .try_for_each(|predicate| -> Result<()> {
                            if predicate.texts.is_empty() {
                            } else if auto_commit && predicate.can_commit {
                                preprocessor.commit(predicate.texts[0].to_owned());

                                rdev::simulate(&EventType::KeyPress(E_Key::Pause)).unwrap();
                                keyboard.text(&predicate.texts[0]).unwrap();
                                rdev::simulate(&EventType::KeyRelease(E_Key::Pause)).unwrap();
                            } else {
                                frontend_tx1.send(GUICmd::Predicate(predicate))?;
                            }

                            Ok(())
                        })?;
                }

                // Update frontend.
                frontend_tx1.send(GUICmd::InputText(curr_input))?;
                frontend_tx1.send(GUICmd::OutputText(output.clone()))?;
                frontend_tx1.send(GUICmd::Update)?;
            }
        }

        // Process preprocessor instructions
        while let Some(command) = preprocessor.pop_queue() {
            match command {
                EventCmd::CommitText(text) => {
                    output.push_str(&text);
                    frontend_tx1.send(GUICmd::OutputText(output.clone()))?;
                    frontend_tx1.send(GUICmd::Update)?;
                }
                EventCmd::Delete(text) => {
                    let step = text.chars().count();
                    (0..(if step > 0 { step } else { 1 })).for_each(|_| {
                        output.pop();
                    });
                }
                EventCmd::Pause => {
                    rdev::simulate(&EventType::KeyPress(E_Key::Pause)).unwrap();
                }
                EventCmd::Resume => {
                    rdev::simulate(&EventType::KeyRelease(E_Key::Pause)).unwrap();
                }
            };
        }

        // Consult the frontend to know if there have some requests.
        frontend_tx1.send(GUICmd::NOP)?;
        match frontend_rx2.recv()? {
            GUICmd::End => break,
            GUICmd::State(state) => {
                idle = state;
                frontend_tx1.send(GUICmd::State(idle))?;
            }
            _ => (),
        }
    }

    // Wait the frontend to end properly.
    frontend_thread.join().unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{frontend::Console, run, Config};
    use afrish::{self, TkPackLayout};
    use rdev::{self, Button, EventType::*, Key::*};
    use std::sync::mpsc::{self, Sender};
    use std::{thread, time::Duration};

    macro_rules! input {
        ( $( $key:expr )*, $delay:expr ) => (
            $(
                thread::sleep($delay);
                rdev::simulate(&KeyPress($key)).unwrap();
                rdev::simulate(&KeyRelease($key)).unwrap();
            )*
        );
    }

    macro_rules! output {
        ( $textfield: expr, $expected: expr ) => {
            thread::sleep(Duration::from_millis(500));

            // A loop to be sure to got something stable
            loop {
                let a = $textfield.get_to_end((1, 0));
                let b = $textfield.get_to_end((1, 0));

                if (a == b) {
                    let content = a.chars().filter(|c| *c != '\0').collect::<String>();
                    let content = content.trim();

                    assert_eq!(content, $expected);
                    break;
                }
            }
        };
    }

    fn start_sandbox(start_point: &str) -> afrish::TkText {
        let root = afrish::trace_with("wish").unwrap();
        root.title("Afrim Test Environment");

        let input_field = afrish::make_text(&root);
        input_field.width(500);
        input_field.height(120);
        input_field.pack().layout();
        input_field.insert((1, 1), start_point);
        root.position(0, 0);
        afrish::tell_wish("wm protocol . WM_DELETE_WINDOW {destroy .};");
        thread::sleep(Duration::from_secs(1));

        input_field
    }

    fn end_sandbox() {
        afrish::end_wish();
    }

    fn start_simulation(signal: Sender<()>) {
        let typing_speed_ms = Duration::from_millis(500);

        // To detect excessive backspace
        const LIMIT: &str = "bbb";

        // Start the sandbox
        let textfield = start_sandbox(LIMIT);

        // Send the ready signal.
        signal.send(()).unwrap();

        rdev::simulate(&MouseMove { x: 100.0, y: 100.0 }).unwrap();
        thread::sleep(typing_speed_ms);
        rdev::simulate(&ButtonPress(Button::Left)).unwrap();
        thread::sleep(typing_speed_ms);
        rdev::simulate(&ButtonRelease(Button::Left)).unwrap();
        thread::sleep(typing_speed_ms);

        input!(KeyU, typing_speed_ms);
        #[cfg(not(feature = "inhibit"))]
        input!(Backspace, typing_speed_ms);
        input!(KeyU KeyU Backspace KeyU, typing_speed_ms);
        input!(
            KeyC Num8 KeyC KeyE KeyD
            KeyU KeyU
            KeyA KeyF Num3, typing_speed_ms);
        input!(
            KeyA KeyF KeyA KeyF
            KeyA KeyF KeyF Num3, typing_speed_ms);
        input!(KeyU KeyU Num3, typing_speed_ms);
        #[cfg(feature = "inhibit")]
        output!(textfield, format!("{LIMIT}çʉ̄ɑ̄ɑɑɑ̄ɑ̄ʉ̄"));
        #[cfg(not(feature = "inhibit"))]
        output!(textfield, format!("{LIMIT}uçʉ̄ɑ̄ɑɑɑ̄ɑ̄ʉ̄"));

        // We verify that the undo (backspace) works as expected
        #[cfg(not(feature = "inhibit"))]
        (0..12).for_each(|_| {
            input!(Backspace, typing_speed_ms);
        });
        #[cfg(feature = "inhibit")]
        (0..13).for_each(|_| {
            input!(Backspace, typing_speed_ms);
        });
        output!(textfield, LIMIT);

        // We verify that the pause/resume works as expected
        rdev::simulate(&KeyPress(ControlLeft)).unwrap();
        rdev::simulate(&KeyPress(ControlRight)).unwrap();
        rdev::simulate(&KeyRelease(ControlRight)).unwrap();
        rdev::simulate(&KeyRelease(ControlLeft)).unwrap();
        input!(KeyU KeyU, typing_speed_ms);

        rdev::simulate(&KeyPress(ControlLeft)).unwrap();
        rdev::simulate(&KeyPress(ControlRight)).unwrap();
        rdev::simulate(&KeyRelease(ControlRight)).unwrap();
        rdev::simulate(&KeyRelease(ControlLeft)).unwrap();
        input!(KeyA KeyF, typing_speed_ms);
        output!(textfield, format!("{LIMIT}uuɑ"));
        input!(Escape, typing_speed_ms);

        // We verify the auto capitalization works as expected
        input!(CapsLock KeyA CapsLock KeyF, typing_speed_ms);
        input!(CapsLock KeyA CapsLock KeyF KeyF, typing_speed_ms);
        input!(KeyA KeyF KeyF, typing_speed_ms);
        output!(textfield, format!("{LIMIT}uuɑαⱭⱭɑɑ"));
        input!(Escape, typing_speed_ms);

        // We verify that the translation work as expected
        input!(KeyH KeyE KeyL KeyL KeyO, typing_speed_ms);
        output!(textfield, format!("{LIMIT}uuɑαⱭⱭɑɑhi"));
        #[cfg(not(feature = "rhai"))]
        input!(Escape KeyH Escape KeyE KeyL KeyL KeyO, typing_speed_ms);
        #[cfg(feature = "rhai")]
        input!(Escape KeyH KeyI, typing_speed_ms);
        output!(textfield, format!("{LIMIT}uuɑαⱭⱭɑɑhihello"));
        input!(Escape, typing_speed_ms);

        // We verify that the predicate selection work as expected
        input!(KeyH KeyE, typing_speed_ms);
        rdev::simulate(&KeyPress(ControlLeft)).unwrap();
        input!(ShiftLeft, typing_speed_ms);
        input!(ShiftRight, typing_speed_ms);
        rdev::simulate(&KeyRelease(ControlLeft)).unwrap();

        input!(KeyA, typing_speed_ms);
        rdev::simulate(&KeyPress(ControlLeft)).unwrap();
        input!(Space, typing_speed_ms);
        rdev::simulate(&KeyRelease(ControlLeft)).unwrap();
        output!(textfield, format!("{LIMIT}uuɑαⱭⱭɑɑhihellohealth"));
        input!(Escape, typing_speed_ms);

        // We verify that we don't have a conflict
        // between the translator and the processor
        input!(KeyV KeyU KeyU KeyE, typing_speed_ms);
        output!(textfield, format!("{LIMIT}uuɑαⱭⱭɑɑhihellohealthvʉe"));

        // Test the idle state from the frontend.
        input!(Escape Num8 KeyS KeyT KeyQ KeyT KeyE Num8, typing_speed_ms);
        input!(Escape, typing_speed_ms);
        rdev::simulate(&KeyPress(ShiftLeft)).unwrap();
        input!(Minus, typing_speed_ms);
        rdev::simulate(&KeyRelease(ShiftLeft)).unwrap();
        input!(KeyS KeyT KeyA KeyT KeyE, typing_speed_ms);
        rdev::simulate(&KeyPress(ShiftLeft)).unwrap();
        input!(Minus, typing_speed_ms);
        rdev::simulate(&KeyRelease(ShiftLeft)).unwrap();

        // End the test
        input!(Escape Num8 KeyE KeyX KeyI KeyT Num8, typing_speed_ms);
        input!(Escape, typing_speed_ms);
        rdev::simulate(&KeyPress(ShiftLeft)).unwrap();
        input!(Minus, typing_speed_ms);
        rdev::simulate(&KeyRelease(ShiftLeft)).unwrap();
        input!(KeyE KeyX KeyI KeyT, typing_speed_ms);
        rdev::simulate(&KeyPress(ShiftLeft)).unwrap();
        input!(Minus, typing_speed_ms);
        rdev::simulate(&KeyRelease(ShiftLeft)).unwrap();

        end_sandbox();
    }

    #[test]
    fn test_afrim() {
        use std::path::Path;

        let (tx, rx) = mpsc::channel();

        // TODO: handle simulation errors.
        let simulation_thread = thread::spawn(move || start_simulation(tx));

        // Wait the ready signal.
        rx.recv().unwrap();

        let test_config = Config::from_file(Path::new("./data/test.toml")).unwrap();
        assert!(run(test_config, Console::default()).is_ok());

        // Wait the simulation to end properly.
        simulation_thread.join().unwrap();
    }
}
