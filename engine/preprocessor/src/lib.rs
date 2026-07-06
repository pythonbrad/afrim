#![deny(missing_docs)]
//! Preprocess keyboard events for an input method.
//!
//! Enables the generation of keyboard event responses from a keyboard input event in an input method
//! engine.
//! The `afrim-preprocessor` crate is built on the top of the [`afrim-memory`](afrim_memory) crate.
//!
//! # Example
//!
//! ```
//! use afrim_preprocessor::{utils::{self, webdriver}, Command, Preprocessor, Node};
//! use std::{collections::VecDeque, rc::Rc};
//!
//! // Prepares the memory.
//! let data = utils::load_data("cc ç");
//! let text_buffer: Node = utils::build_map(data);
//! let memory = Rc::new(text_buffer);
//!
//! // Builds the preprocessor.
//! let mut preprocessor = Preprocessor::new(memory, 8);
//!
//! // Process an input.
//! let input = "cc";
//! webdriver::send_keys(input)
//!     .into_iter()
//!     .for_each(|event| {
//!         match event {
//!             // Triggers the generated keyboard input event.
//!             webdriver::Event::Keyboard(event) => preprocessor.process(event),
//!             _ => unimplemented!(),
//!         };
//!     });
//!
//! // Now let's look at the generated commands.
//! // The expected results without `inhibit` feature.
//! #[cfg(not(feature = "inhibit"))]
//! let mut expecteds = VecDeque::from(vec![
//!     Command::Pause,
//!     Command::Delete("c".to_string()),
//!     Command::Delete("c".to_string()),
//!     Command::CommitText("ç".to_owned()),
//!     Command::Resume,
//! ]);
//!
//! // The expected results with `inhibit` feature.
//! #[cfg(feature = "inhibit")]
//! let mut expecteds = VecDeque::from(vec![
//!     Command::Pause,
//!     Command::Delete("c".to_string()),
//!     Command::Resume,
//!     Command::Pause,
//!     Command::Delete("c".to_string()),
//!     Command::CommitText("ç".to_owned()),
//!     Command::Resume,
//! ]);
//!
//! // Verification.
//! while let Some(command) = preprocessor.pop_queue() {
//!     assert_eq!(command, expecteds.pop_front().unwrap());
//! }
//! assert_eq!(expecteds.is_empty(), true);
//! ```
//! **Note**: When dealing with non latin languages. The `inhibit` feature allows for the removal of
//! unwanted characters typically latin characters, as much as posssible.

mod message;

pub use crate::message::Command;
use afrim_memory::Cursor;
pub use afrim_memory::Node;
pub use keyboard_types::{Key, KeyState, KeyboardEvent, NamedKey};
use std::{collections::VecDeque, rc::Rc};

/// Utilities.
pub mod utils {
    pub use afrim_memory::utils::{build_map, load_data};
    pub use keyboard_types::webdriver;
}

/// The main structure of the preprocessor.
#[derive(Debug)]
pub struct Preprocessor {
    cursor: Cursor,
    queue: VecDeque<Command>,
}

impl Preprocessor {
    /// Initializes a new preprocessor.
    ///
    /// The preprocessor needs a memory to operate. You have two options to build this memory.
    /// - Use the [`afrim-memory`] crate.
    /// - Use the [`utils`] module.
    ///
    /// It also needs you set the capacity of his cursor. We recommend to set a capacity equal
    /// or greater than N times the maximun sequence length that you want to handle.
    /// Where N is the number of sequences that you want track in the cursor.
    ///
    /// Note that the cursor is the internal memory of the `afrim_preprocessor`.
    ///
    /// # Example
    ///
    /// ```
    /// use afrim_preprocessor::{Preprocessor, utils};
    /// use std::rc::Rc;
    ///
    /// // We prepare the memory.
    /// let data = utils::load_data("uuaf3    ʉ̄ɑ̄");
    /// let text_buffer = utils::build_map(data);
    /// let memory = Rc::new(text_buffer);
    ///
    /// // We initialize our preprocessor.
    /// let preprocessor = Preprocessor::new(memory, 8);
    /// ```
    pub fn new(memory: Rc<Node>, buffer_size: usize) -> Self {
        let cursor = Cursor::new(memory, buffer_size);
        let queue = VecDeque::with_capacity(15);

        Self { cursor, queue }
    }

    // Cancel the previous operation.
    fn rollback(&mut self) -> bool {
        let (_, _, _curr_char) = self.cursor.state();

        if let Some(out) = self.cursor.undo() {
            self.queue.push_back(Command::Delete(out));

            // Clear the remaining code
            while let (None, 1.., ..) = self.cursor.state() {
                self.cursor.undo();
            }

            if let (Some(_in), ..) = self.cursor.state() {
                self.queue.push_back(Command::CommitText(_in));
            }

            true
        } else {
            #[cfg(not(feature = "inhibit"))]
            self.queue
                .push_back(Command::Delete(_curr_char.to_string()));
            self.cursor.resume();

            false
        }
    }

    /// Preprocess the keyboard input event and returns infos on his internal changes (change on
    /// the cursor and/or something to commit).
    ///
    /// It's useful when you process keyboard input events in bulk. Whether there is something that
    /// you want to do based on this information, you can decide how to continue.
    ///
    /// # Example
    ///
    /// ```
    /// use afrim_preprocessor::{Command, Preprocessor, utils};
    /// use keyboard_types::{Key::Character, KeyboardEvent};
    /// use std::{collections::VecDeque, rc::Rc};
    ///
    /// // We prepare the memory.
    /// let data = utils::load_data("i3  ī");
    /// let text_buffer = utils::build_map(data);
    /// let memory = Rc::new(text_buffer);
    ///
    /// let mut preprocessor = Preprocessor::new(memory, 8);
    ///
    /// // We process the input.
    /// // let input = "si3";
    ///
    /// let info = preprocessor.process(KeyboardEvent {
    ///     key: Character("s".to_string()),
    ///     ..Default::default()
    /// });
    /// assert_eq!(info, (true, false));
    ///
    /// let info = preprocessor.process(KeyboardEvent {
    ///     key: Character("i".to_string()),
    ///     ..Default::default()
    /// });
    /// assert_eq!(info, (true, false));
    ///
    /// let info = preprocessor.process(KeyboardEvent {
    ///     key: Character("3".to_string()),
    ///     ..Default::default()
    /// });
    /// assert_eq!(info, (true, true));
    ///
    /// // The input inside the preprocessor.
    /// assert_eq!(preprocessor.get_input(), "si3".to_owned());
    ///
    /// // The generated commands.
    /// // The expected results without inhibit feature.
    /// #[cfg(not(feature = "inhibit"))]
    /// let mut expecteds = VecDeque::from(vec![
    ///     Command::Pause,
    ///     Command::Delete("3".to_string()),
    ///     Command::Delete("i".to_string()),
    ///     Command::CommitText("ī".to_owned()),
    ///     Command::Resume,
    /// ]);
    ///
    /// // The expected results with inhibit feature.
    /// #[cfg(feature = "inhibit")]
    /// let mut expecteds = VecDeque::from(vec![
    ///     Command::Pause,
    ///     Command::Delete("s".to_string()),
    ///     Command::Resume,
    ///     Command::Pause,
    ///     Command::Delete("i".to_string()),
    ///     Command::Resume,
    ///     Command::Pause,
    ///     Command::Delete("3".to_string()),
    ///     Command::CommitText("ī".to_owned()),
    ///     Command::Resume,
    /// ]);
    ///
    /// // Verification.
    /// while let Some(command) = preprocessor.pop_queue() {
    ///     assert_eq!(command, expecteds.pop_front().unwrap());
    /// }
    /// assert_eq!(expecteds.is_empty(), true);
    /// ```
    pub fn process(&mut self, event: KeyboardEvent) -> (bool, bool) {
        let (mut changed, mut committed) = (false, false);

        match (event.state, event.key) {
            (KeyState::Down, Key::Named(NamedKey::Backspace)) => {
                #[cfg(not(feature = "inhibit"))]
                {
                    self.pause();
                    committed = self.rollback();
                    self.resume();
                }
                #[cfg(feature = "inhibit")]
                self.cursor.clear();
                changed = true;
            }
            (KeyState::Down, Key::Character(character))
                if character
                    .chars()
                    .next()
                    .map(|e| e.is_alphanumeric() || e.is_ascii_punctuation())
                    .unwrap_or(false) =>
            {
                #[cfg(feature = "inhibit")]
                self.pause();
                #[cfg(feature = "inhibit")]
                self.queue.push_back(Command::Delete(character.clone()));

                let character = character.chars().next().unwrap();

                if let Some(_in) = self.cursor.hit(character) {
                    #[cfg(not(feature = "inhibit"))]
                    self.pause();
                    let mut prev_cursor = self.cursor.clone();
                    prev_cursor.undo();
                    #[cfg(not(feature = "inhibit"))]
                    self.queue.push_back(Command::Delete(character.to_string()));

                    // Remove the remaining code
                    while let (None, 1.., _c) = prev_cursor.state() {
                        prev_cursor.undo();
                        #[cfg(not(feature = "inhibit"))]
                        self.queue.push_back(Command::Delete(_c.to_string()));
                    }

                    if let (Some(out), ..) = prev_cursor.state() {
                        self.queue.push_back(Command::Delete(out))
                    }

                    self.queue.push_back(Command::CommitText(_in));
                    #[cfg(not(feature = "inhibit"))]
                    self.resume();
                    committed = true;
                };

                #[cfg(feature = "inhibit")]
                self.resume();
                changed = true;
            }
            (KeyState::Down, Key::Named(NamedKey::Shift) | Key::Named(NamedKey::CapsLock)) => (),
            (KeyState::Down, _) => {
                self.cursor.clear();
                changed = true;
            }
            _ => (),
        };

        (changed, committed)
    }

    /// Commit a text.
    ///
    /// Generate a command to ensure the commitment of this text.
    /// Useful when you want deal with auto-completion.
    ///
    /// **Note**: Before any commitment, the preprocessor make sure to discard the current input.
    ///
    /// # Example
    ///
    /// ```
    /// use afrim_preprocessor::{Command, Preprocessor, utils, KeyboardEvent, Key::Character};
    /// use std::{collections::VecDeque, rc::Rc};
    ///
    /// // We prepare the memory.
    /// let data = utils::load_data("i3  ī");
    /// let text_buffer = utils::build_map(data);
    /// let memory = Rc::new(text_buffer);
    ///
    /// let mut preprocessor = Preprocessor::new(memory, 8);
    ///
    /// // We process the input.
    /// // let input = "si3";
    /// preprocessor.process(KeyboardEvent {
    ///     key: Character("s".to_string()),
    ///     ..Default::default()
    /// });
    ///
    /// preprocessor.commit("sī".to_owned());
    ///
    /// // The generated commands.
    /// // The expected results without inhibit feature.
    /// #[cfg(not(feature = "inhibit"))]
    /// let mut expecteds = VecDeque::from(vec![
    ///     Command::Pause,
    ///     Command::Delete("s".to_string()),
    ///     Command::CommitText("sī".to_owned()),
    ///     Command::Resume,
    /// ]);
    ///
    /// // The expected results with inhibit feature.
    /// #[cfg(feature = "inhibit")]
    /// let mut expecteds = VecDeque::from(vec![
    ///     Command::Pause,
    ///     Command::Delete("s".to_string()),
    ///     Command::Resume,
    ///     Command::Pause,
    ///     Command::CommitText("sī".to_owned()),
    ///     Command::Resume,
    /// ]);
    ///
    /// // Verification.
    /// while let Some(command) = preprocessor.pop_queue() {
    ///     assert_eq!(command, expecteds.pop_front().unwrap());
    /// }
    /// assert_eq!(expecteds.is_empty(), true);
    /// ```
    pub fn commit(&mut self, text: String) {
        self.pause();

        while !self.cursor.is_empty() {
            self.rollback();
        }
        #[cfg(feature = "inhibit")]
        self.cursor.clear();
        self.queue.push_back(Command::CommitText(text));
        self.resume();
        // We clear the buffer
        self.cursor.clear();
    }

    // Pauses the keyboard event listerner.
    fn pause(&mut self) {
        self.queue.push_back(Command::Pause);
    }

    // Resumes the keyboard event listener.
    fn resume(&mut self) {
        self.queue.push_back(Command::Resume);
    }

    /// Returns the input present in the internal memory.
    ///
    /// It's always useful to know what is inside the memory of the preprocessor for debugging.
    /// **Note**: The input inside the preprocessor is not always the same than the original because
    /// of the limited capacity of his internal cursor.
    ///
    /// # Example
    ///
    /// ```
    /// use afrim_preprocessor::{Command, Preprocessor, utils::{self, webdriver}};
    /// use std::{collections::VecDeque, rc::Rc};
    ///
    /// // We prepare the memory.
    /// let data = utils::load_data("i3  ī");
    /// let text_buffer = utils::build_map(data);
    /// let memory = Rc::new(text_buffer);
    ///
    /// let mut preprocessor = Preprocessor::new(memory, 4);
    ///
    /// // We process the input.
    /// let input = "si3";
    /// webdriver::send_keys(input)
    ///     .into_iter()
    ///     .for_each(|event| {
    ///         match event {
    ///             // Triggers the generated keyboard input event.
    ///             webdriver::Event::Keyboard(event) => preprocessor.process(event),
    ///             _ => unimplemented!(),
    ///         };
    ///     });
    ///
    /// // The input inside the processor.
    /// assert_eq!(preprocessor.get_input(), "si3".to_owned());
    pub fn get_input(&self) -> String {
        self.cursor
            .to_sequence()
            .into_iter()
            .filter(|c| *c != '\0')
            .collect::<String>()
    }

    /// Returns the next command to be executed.
    ///
    /// The next command is dropped from the queue and can't be returned anymore.
    ///
    /// # Example
    ///
    /// ```
    /// use afrim_preprocessor::{Command, Preprocessor, utils};
    /// use std::{collections::VecDeque, rc::Rc};
    ///
    /// // We prepare the memory.
    /// let text_buffer = utils::build_map(vec![]);
    /// let memory = Rc::new(text_buffer);
    ///
    /// let mut preprocessor = Preprocessor::new(memory, 8);
    /// preprocessor.commit("hello".to_owned());
    ///
    /// // The expected results.
    /// let mut expecteds = VecDeque::from(vec![
    ///     Command::Pause,
    ///     Command::CommitText("hello".to_owned()),
    ///     Command::Resume,
    /// ]);
    ///
    /// // Verification.
    /// while let Some(command) = preprocessor.pop_queue() {
    ///     assert_eq!(command, expecteds.pop_front().unwrap());
    /// }
    /// assert_eq!(expecteds.is_empty(), true);
    pub fn pop_queue(&mut self) -> Option<Command> {
        self.queue.pop_front()
    }

    /// Clears the queue.
    ///
    /// # Example
    ///
    /// ```
    /// use afrim_preprocessor::{Preprocessor, utils};
    /// use std::rc::Rc;
    ///
    /// let data =
    /// utils::load_data("n* ŋ");
    /// let text_buffer = utils::build_map(data);
    /// let memory = Rc::new(text_buffer);
    ///
    /// let mut preprocessor = Preprocessor::new(memory, 8);
    /// preprocessor.commit("hi".to_owned());
    /// preprocessor.clear_queue();
    ///
    /// assert_eq!(preprocessor.pop_queue(), None);
    /// ```
    pub fn clear_queue(&mut self) {
        self.queue.clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::message::Command;
    use crate::Preprocessor;
    use crate::{
        utils::{self, webdriver},
        Key::{Character, Named},
        KeyboardEvent, NamedKey, Node,
    };
    use std::collections::VecDeque;

    #[test]
    fn test_process() {
        use std::rc::Rc;

        let data = utils::load_data("ccced ç\ncc ç");
        let memory = utils::build_map(data);
        let mut preprocessor = Preprocessor::new(Rc::new(memory), 8);
        webdriver::send_keys("ccced").into_iter().for_each(|e| {
            match e {
                webdriver::Event::Keyboard(e) => preprocessor.process(e),
                _ => unimplemented!(),
            };
        });
        #[cfg(not(feature = "inhibit"))]
        let mut expecteds = VecDeque::from(vec![
            // c c
            Command::Pause,
            Command::Delete("c".to_string()),
            Command::Delete("c".to_string()),
            Command::CommitText("ç".to_owned()),
            Command::Resume,
            // c e d
            Command::Pause,
            Command::Delete("d".to_string()),
            Command::Delete("e".to_string()),
            Command::Delete("c".to_string()),
            Command::Delete("ç".to_string()),
            Command::CommitText("ç".to_owned()),
            Command::Resume,
        ]);
        #[cfg(feature = "inhibit")]
        let mut expecteds = VecDeque::from(vec![
            // c c
            Command::Pause,
            Command::Delete("c".to_string()),
            Command::Resume,
            Command::Pause,
            Command::Delete("c".to_string()),
            Command::CommitText("ç".to_owned()),
            Command::Resume,
            // c e d
            Command::Pause,
            Command::Delete("c".to_string()),
            Command::Resume,
            Command::Pause,
            Command::Delete("e".to_string()),
            Command::Resume,
            Command::Pause,
            Command::Delete("d".to_string()),
            Command::Delete("ç".to_string()),
            Command::CommitText("ç".to_owned()),
            Command::Resume,
        ]);

        while let Some(command) = preprocessor.pop_queue() {
            assert_eq!(command, expecteds.pop_front().unwrap());
        }
        assert!(expecteds.is_empty());
    }

    #[test]
    fn test_commit() {
        let mut preprocessor = Preprocessor::new(Node::default().into(), 8);
        preprocessor.process(KeyboardEvent {
            key: Character("a".to_owned()),
            ..Default::default()
        });
        preprocessor.commit("word".to_owned());

        let mut expecteds = VecDeque::from(vec![
            Command::Pause,
            Command::Delete("a".to_string()),
            #[cfg(feature = "inhibit")]
            Command::Resume,
            #[cfg(feature = "inhibit")]
            Command::Pause,
            Command::CommitText("word".to_owned()),
            Command::Resume,
        ]);

        while let Some(command) = preprocessor.pop_queue() {
            assert_eq!(command, expecteds.pop_front().unwrap());
        }
        assert!(expecteds.is_empty());
    }

    #[test]
    fn test_rollback() {
        use std::rc::Rc;

        let data = utils::load_data("ccced ç\ncc ç");
        let memory = utils::build_map(data);
        let mut preprocessor = Preprocessor::new(Rc::new(memory), 8);
        let backspace_event = KeyboardEvent {
            key: Named(NamedKey::Backspace),
            ..Default::default()
        };

        webdriver::send_keys("ccced").into_iter().for_each(|e| {
            match e {
                webdriver::Event::Keyboard(e) => preprocessor.process(e),
                _ => unimplemented!(),
            };
        });

        preprocessor.clear_queue();
        assert_eq!(preprocessor.get_input(), "ccced".to_owned());
        preprocessor.process(backspace_event.clone());
        #[cfg(not(feature = "inhibit"))]
        assert_eq!(preprocessor.get_input(), "cc".to_owned());
        #[cfg(not(feature = "inhibit"))]
        preprocessor.process(backspace_event);
        assert_eq!(preprocessor.get_input(), "".to_owned());

        #[cfg(not(feature = "inhibit"))]
        let mut expecteds = VecDeque::from(vec![
            Command::Pause,
            Command::Delete("ç".to_string()),
            Command::CommitText("ç".to_owned()),
            Command::Resume,
            Command::Pause,
            Command::Delete("ç".to_string()),
            Command::Resume,
        ]);

        // NOTE: The inhibit feature don't support rollback
        #[cfg(feature = "inhibit")]
        let mut expecteds = VecDeque::from(vec![]);

        while let Some(command) = preprocessor.pop_queue() {
            assert_eq!(command, expecteds.pop_front().unwrap());
        }
        assert!(expecteds.is_empty());
    }

    #[test]
    fn test_advanced() {
        use std::rc::Rc;

        let data = include_str!("../data/sample.txt");
        let data = utils::load_data(data);
        let memory = utils::build_map(data);
        let mut preprocessor = Preprocessor::new(Rc::new(memory), 64);

        webdriver::send_keys(
            "u\u{E003}uu\u{E003}uc_ceduuaf3afafaff3uu3\
            \u{E003}\u{E003}\u{E003}\u{E003}\u{E003}\u{E003}\u{E003}\u{E003}\u{E003}\u{E003}\u{E003}\u{E003}"
        ).into_iter().for_each(|e| {
            match e {
                webdriver::Event::Keyboard(e) => preprocessor.process(e),
                _ => unimplemented!(),
            };
        });

        #[cfg(not(feature = "inhibit"))]
        let mut expecteds = VecDeque::from(vec![
            // Process
            // u backspace
            Command::Pause,
            Command::Delete("u".to_string()),
            Command::Resume,
            // u u backspace
            Command::Pause,
            Command::Delete("u".to_string()),
            Command::Delete("u".to_string()),
            Command::CommitText("ʉ".to_owned()),
            Command::Resume,
            Command::Pause,
            Command::Delete("ʉ".to_string()),
            Command::Resume,
            // u
            // NOP
            // c _
            Command::Pause,
            Command::Delete("_".to_string()),
            Command::Delete("c".to_string()),
            Command::CommitText("ç".to_owned()),
            Command::Resume,
            // c e d
            Command::Pause,
            Command::Delete("d".to_string()),
            Command::Delete("e".to_string()),
            Command::Delete("c".to_string()),
            Command::Delete("ç".to_string()),
            Command::CommitText("ç".to_owned()),
            Command::Resume,
            // u u
            Command::Pause,
            Command::Delete("u".to_string()),
            Command::Delete("u".to_string()),
            Command::CommitText("ʉ".to_owned()),
            Command::Resume,
            // a f 3
            Command::Pause,
            Command::Delete("3".to_string()),
            Command::Delete("f".to_string()),
            Command::Delete("a".to_string()),
            Command::Delete("ʉ".to_string()),
            Command::CommitText("ʉ\u{304}ɑ\u{304}".to_owned()),
            Command::Resume,
            // a f
            Command::Pause,
            Command::Delete("f".to_string()),
            Command::Delete("a".to_string()),
            Command::CommitText("ɑ".to_owned()),
            Command::Resume,
            // a f
            Command::Pause,
            Command::Delete("f".to_string()),
            Command::Delete("a".to_string()),
            Command::CommitText("ɑ".to_owned()),
            Command::Resume,
            // a f
            Command::Pause,
            Command::Delete("f".to_string()),
            Command::Delete("a".to_string()),
            Command::CommitText("ɑ".to_owned()),
            Command::Resume,
            // f
            Command::Pause,
            Command::Delete("f".to_string()),
            Command::Delete("ɑ".to_string()),
            Command::CommitText("ɑɑ".to_owned()),
            Command::Resume,
            // 3
            Command::Pause,
            Command::Delete("3".to_string()),
            Command::Delete("ɑɑ".to_string()),
            Command::CommitText("ɑ\u{304}ɑ\u{304}".to_owned()),
            Command::Resume,
            // uu
            Command::Pause,
            Command::Delete("u".to_string()),
            Command::Delete("u".to_string()),
            Command::CommitText("ʉ".to_owned()),
            Command::Resume,
            // 3
            Command::Pause,
            Command::Delete("3".to_string()),
            Command::Delete("ʉ".to_string()),
            Command::CommitText("ʉ\u{304}".to_owned()),
            Command::Resume,
            // Rollback
            Command::Pause,
            Command::Delete("ʉ\u{304}".to_string()),
            Command::CommitText("ʉ".to_owned()),
            Command::Resume,
            Command::Pause,
            Command::Delete("ʉ".to_string()),
            Command::Resume,
            Command::Pause,
            Command::Delete("ɑ\u{304}ɑ\u{304}".to_string()),
            Command::CommitText("ɑɑ".to_owned()),
            Command::Resume,
            Command::Pause,
            Command::Delete("ɑɑ".to_string()),
            Command::CommitText("ɑ".to_owned()),
            Command::Resume,
            Command::Pause,
            Command::Delete("ɑ".to_string()),
            Command::Resume,
            Command::Pause,
            Command::Delete("ɑ".to_string()),
            Command::Resume,
            Command::Pause,
            Command::Delete("ɑ".to_string()),
            Command::Resume,
            Command::Pause,
            Command::Delete("ʉ\u{304}ɑ\u{304}".to_string()),
            Command::CommitText("ʉ".to_owned()),
            Command::Resume,
            Command::Pause,
            Command::Delete("ʉ".to_string()),
            Command::Resume,
            Command::Pause,
            Command::Delete("ç".to_string()),
            Command::CommitText("ç".to_owned()),
            Command::Resume,
            Command::Pause,
            Command::Delete("ç".to_string()),
            Command::Resume,
            // yes, the buffer is empty, but we want leave the user use the backspace.
            Command::Pause,
            Command::Delete("\0".to_string()),
            Command::Resume,
        ]);
        #[cfg(feature = "inhibit")]
        let mut expecteds = VecDeque::from(vec![
            // Process
            // u backspace
            Command::Pause,
            Command::Delete("u".to_string()),
            Command::Resume,
            // u u backspace
            Command::Pause,
            Command::Delete("u".to_string()),
            Command::Resume,
            Command::Pause,
            Command::Delete("u".to_string()),
            Command::CommitText("ʉ".to_owned()),
            Command::Resume,
            // u
            Command::Pause,
            Command::Delete("u".to_string()),
            Command::Resume,
            // c _
            Command::Pause,
            Command::Delete("c".to_string()),
            Command::Resume,
            Command::Pause,
            Command::Delete("_".to_string()),
            Command::CommitText("ç".to_owned()),
            Command::Resume,
            // c e d
            Command::Pause,
            Command::Delete("c".to_string()),
            Command::Resume,
            Command::Pause,
            Command::Delete("e".to_string()),
            Command::Resume,
            Command::Pause,
            Command::Delete("d".to_string()),
            Command::Delete("ç".to_string()),
            Command::CommitText("ç".to_owned()),
            Command::Resume,
            // u u
            Command::Pause,
            Command::Delete("u".to_string()),
            Command::Resume,
            Command::Pause,
            Command::Delete("u".to_string()),
            Command::CommitText("ʉ".to_owned()),
            Command::Resume,
            // a f 3
            Command::Pause,
            Command::Delete("a".to_string()),
            Command::Resume,
            Command::Pause,
            Command::Delete("f".to_string()),
            Command::Resume,
            Command::Pause,
            Command::Delete("3".to_string()),
            Command::Delete("ʉ".to_string()),
            Command::CommitText("ʉ\u{304}ɑ\u{304}".to_owned()),
            Command::Resume,
            // a f
            Command::Pause,
            Command::Delete("a".to_string()),
            Command::Resume,
            Command::Pause,
            Command::Delete("f".to_string()),
            Command::CommitText("ɑ".to_owned()),
            Command::Resume,
            // a f
            Command::Pause,
            Command::Delete("a".to_string()),
            Command::Resume,
            Command::Pause,
            Command::Delete("f".to_string()),
            Command::CommitText("ɑ".to_owned()),
            Command::Resume,
            // a f
            Command::Pause,
            Command::Delete("a".to_string()),
            Command::Resume,
            Command::Pause,
            Command::Delete("f".to_string()),
            Command::CommitText("ɑ".to_owned()),
            Command::Resume,
            // f
            Command::Pause,
            Command::Delete("f".to_string()),
            Command::Delete("ɑ".to_string()),
            Command::CommitText("ɑɑ".to_owned()),
            Command::Resume,
            // 3
            Command::Pause,
            Command::Delete("3".to_string()),
            Command::Delete("ɑɑ".to_string()),
            Command::CommitText("ɑ\u{304}ɑ\u{304}".to_owned()),
            Command::Resume,
            // uu
            Command::Pause,
            Command::Delete("u".to_string()),
            Command::Resume,
            Command::Pause,
            Command::Delete("u".to_string()),
            Command::CommitText("ʉ".to_owned()),
            Command::Resume,
            // 3
            Command::Pause,
            Command::Delete("3".to_string()),
            Command::Delete("ʉ".to_owned()),
            Command::CommitText("ʉ\u{304}".to_owned()),
            Command::Resume,
            // Rollback
            // NOTE: The inhibit feature don't support rollback
        ]);

        while let Some(command) = preprocessor.pop_queue() {
            let c = expecteds.pop_front().unwrap();
            assert_eq!(command, c);
        }
        assert!(expecteds.is_empty());
    }
}
