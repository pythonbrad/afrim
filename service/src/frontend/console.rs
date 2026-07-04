#![deny(missing_docs)]
//! Console frontend interface for the Afrim.
//!

use super::{message::Command, Frontend, Predicate};
use anyhow::{anyhow, Result};
use std::sync::mpsc::{Receiver, Sender};

/// Cli frontent interface.
#[derive(Default)]
pub struct Console {
    page_size: usize,
    predicates: Vec<Predicate>,
    current_predicate_id: usize,
    input: String,
    output: String,
    idle_state: bool,
    tx: Option<Sender<Command>>,
    rx: Option<Receiver<Command>>,
}

impl Frontend for Console {
    fn init(&mut self, tx: Sender<Command>, rx: Receiver<Command>) -> Result<()> {
        self.tx = Some(tx);
        self.rx = Some(rx);

        Ok(())
    }

    fn listen(&mut self) -> Result<()> {
        if self.tx.as_ref().and(self.rx.as_ref()).is_none() {
            return Err(anyhow!("you should config the channel first!"));
        }

        let tx = self.tx.clone().unwrap();

        loop {
            let command = self.rx.as_ref().unwrap().recv()?;
            match command {
                Command::InputText(input) => self.set_input_text(input.to_owned()),
                Command::OutputText(output) => self.set_output_text(output.to_owned()),
                Command::PageSize(size) => self.set_max_predicates(size),
                Command::State(state) => self.set_state(state),
                Command::Predicate(predicate) => self.add_predicate(predicate.to_owned()),
                Command::Update => self.display(),
                Command::Clear => self.clear(),
                Command::SelectPreviousPredicate => self.select_previous_predicate(),
                Command::SelectNextPredicate => self.select_next_predicate(),
                Command::SelectedPredicate => {
                    if let Some(predicate) = self.get_selected_predicate() {
                        tx.send(Command::Predicate(predicate.to_owned()))?;
                    } else {
                        tx.send(Command::NoPredicate)?;
                    }
                }
                Command::NOP => {
                    match self.input.as_str() {
                        // `_state_` is reserved to test the idle state from the console frontend,
                        // since there is no way to toggle it.
                        "_state_" if !self.idle_state => {
                            tx.send(Command::State(true))?;
                        }
                        "_state_" if self.idle_state => {
                            tx.send(Command::State(false))?;
                            self.input = String::default();
                            self.output = String::default();
                        }
                        "_exit_" => {
                            tx.send(Command::End)?;

                            return Ok(());
                        }
                        _ => {
                            tx.send(Command::NOP)?;
                        }
                    };
                }
                Command::End => {
                    tx.send(Command::End)?;

                    return Ok(());
                }
                _ => (),
            }
        }
    }
}

impl Console {
    fn display(&mut self) {
        // Input
        println!("input: {}", self.input);

        // Output
        println!("output: {}", self.output);

        // Predicates
        let page_size = std::cmp::min(self.page_size, self.predicates.len());
        println!(
            "Predicates: {}",
            self.predicates
                .iter()
                .enumerate()
                .chain(self.predicates.iter().enumerate())
                .skip(self.current_predicate_id)
                .take(page_size)
                .map(|(id, predicate)| {
                    format!(
                        "{}{}. {} ~{}\t ",
                        if id == self.current_predicate_id {
                            "*"
                        } else {
                            ""
                        },
                        id + 1,
                        predicate.texts[0],
                        predicate.remaining_code
                    )
                })
                .collect::<Vec<_>>()
                .join("\t")
        );
    }

    fn clear(&mut self) {
        self.predicates.clear();
        self.current_predicate_id = 0;
        self.input = String::default();
        self.output = String::default();
    }

    fn set_max_predicates(&mut self, size: usize) {
        self.page_size = size;
        self.predicates = Vec::with_capacity(size);
    }

    fn set_input_text(&mut self, text: String) {
        self.input = text;
    }

    fn set_output_text(&mut self, text: String) {
        self.output = text;
    }

    fn add_predicate(&mut self, predicate: Predicate) {
        predicate
            .texts
            .iter()
            .filter(|text| !text.is_empty())
            .for_each(|text| {
                let mut predicate = predicate.clone();
                predicate.texts = vec![text.to_owned()];

                self.predicates.push(predicate);
            });
    }

    fn select_previous_predicate(&mut self) {
        if self.predicates.is_empty() {
            return;
        };

        self.current_predicate_id =
            (self.current_predicate_id + self.predicates.len() - 1) % self.predicates.len();
        self.display();
    }

    fn select_next_predicate(&mut self) {
        if self.predicates.is_empty() {
            return;
        };

        self.current_predicate_id = (self.current_predicate_id + 1) % self.predicates.len();
        self.display();
    }

    fn get_selected_predicate(&self) -> Option<&Predicate> {
        self.predicates.get(self.current_predicate_id)
    }

    fn set_state(&mut self, state: bool) {
        self.idle_state = state;
        let state = if state { "paused" } else { "resumed" };

        println!("state: {state}");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_console() {
        use crate::frontend::{Command, Console, Frontend, Predicate};
        use std::sync::mpsc;
        use std::thread;

        let mut console = Console::default();
        assert!(console.listen().is_err());
        let (tx1, rx1) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();

        let console_thread = thread::spawn(move || {
            assert!(console.init(tx2, rx1).is_ok());
            console.listen().unwrap();
        });

        tx1.send(Command::NOP).unwrap();
        assert_eq!(rx2.recv().unwrap(), Command::NOP);

        tx1.send(Command::PageSize(10)).unwrap();

        tx1.send(Command::InputText("ngaf3".to_owned())).unwrap();
        tx1.send(Command::OutputText("ngɑ̄".to_owned())).unwrap();
        tx1.send(Command::Update).unwrap();

        tx1.send(Command::Clear).unwrap();
        tx1.send(Command::Update).unwrap();

        tx1.send(Command::InputText("he".to_owned())).unwrap();
        tx1.send(Command::OutputText("he".to_owned())).unwrap();
        tx1.send(Command::Predicate(Predicate {
            code: "hell".to_owned(),
            remaining_code: "llo".to_owned(),
            texts: vec!["hello".to_owned()],
            can_commit: false,
        }))
        .unwrap();
        tx1.send(Command::Predicate(Predicate {
            code: "helip".to_owned(),
            remaining_code: "lip".to_owned(),
            texts: vec![],
            can_commit: false,
        }))
        .unwrap();
        tx1.send(Command::Predicate(Predicate {
            code: "helio".to_owned(),
            remaining_code: "s".to_owned(),
            texts: vec!["".to_owned()],
            can_commit: false,
        }))
        .unwrap();
        tx1.send(Command::Predicate(Predicate {
            code: "heal".to_owned(),
            remaining_code: "al".to_owned(),
            texts: vec!["health".to_owned()],
            can_commit: false,
        }))
        .unwrap();
        tx1.send(Command::Update).unwrap();

        tx1.send(Command::SelectPreviousPredicate).unwrap();
        tx1.send(Command::SelectedPredicate).unwrap();
        assert_eq!(
            rx2.recv().unwrap(),
            Command::Predicate(Predicate {
                code: "heal".to_owned(),
                remaining_code: "al".to_owned(),
                texts: vec!["health".to_owned()],
                can_commit: false
            })
        );

        tx1.send(Command::SelectNextPredicate).unwrap();
        tx1.send(Command::SelectedPredicate).unwrap();
        assert_eq!(
            rx2.recv().unwrap(),
            Command::Predicate(Predicate {
                code: "hell".to_owned(),
                remaining_code: "llo".to_owned(),
                texts: vec!["hello".to_owned()],
                can_commit: false
            })
        );

        tx1.send(Command::Clear).unwrap();
        tx1.send(Command::SelectPreviousPredicate).unwrap();
        tx1.send(Command::SelectNextPredicate).unwrap();
        tx1.send(Command::SelectedPredicate).unwrap();
        assert_eq!(rx2.recv().unwrap(), Command::NoPredicate);

        // End of communication.
        tx1.send(Command::InputText("_exit_".to_owned())).unwrap();
        tx1.send(Command::NOP).unwrap();
        assert_eq!(rx2.recv().unwrap(), Command::End);

        // Wait the console to end properly.
        console_thread.join().unwrap();
        assert!(tx1.send(Command::NOP).is_err() && rx2.recv().is_err());
    }

    // Test to increase the coverage.
    #[test]
    fn test_console_optional() {
        use crate::frontend::{Command, Console, Frontend};
        use std::sync::mpsc;
        use std::thread;

        let mut console = Console::default();
        let (tx1, rx1) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();

        let console_thread = thread::spawn(move || {
            console.init(tx2, rx1).unwrap();
            console.listen().unwrap();
        });

        tx1.send(Command::InputText("_state_".to_owned())).unwrap();
        tx1.send(Command::NOP).unwrap();
        assert_eq!(rx2.recv().unwrap(), Command::State(true));
        tx1.send(Command::State(true)).unwrap();
        tx1.send(Command::NOP).unwrap();
        assert_eq!(rx2.recv().unwrap(), Command::State(false));
        tx1.send(Command::State(false)).unwrap();
        tx1.send(Command::NOP).unwrap();
        assert_eq!(rx2.recv().unwrap(), Command::NOP);

        tx1.send(Command::End).unwrap();
        assert_eq!(rx2.recv().unwrap(), Command::End);

        // Wait the console to end properly.
        console_thread.join().unwrap();
    }
}
