//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to deal with logging.

This module contains all the code related with logging. Note that logs are generated here,
but it's up to you to save them somewhere.
!*/

use log::{info, warn, error};

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

/// This struct is what takes care of logging everything in this crate.
///
/// The way it works is, you initialize it, then just use his functions to log message.
/// Logging is done in a separate thread, so performance impact should be minimal, and
/// the logging thread can continue working in case of crash, so it can correctly log 
/// the crash, then stop.
pub struct Logger (Arc<Mutex<Sender<LogLevel>>>);

/// This enum holds the different logging levels we have.
pub enum LogLevel {
	Info(String),
	Warning(String),
	Error(String),
	Stop,
}

/// Implementation of `Logger`.
impl Logger {

	/// This function initialize the entire logger as a multithread logger.
	pub fn init_logger() -> Self {
		let (sender, receiver) = channel();
	    thread::spawn(move || { logger(receiver); });
	    Self(Arc::new(Mutex::new(sender)))
	}
	
	/// This function sends the provided `LogLevel` to the logger thread, so it can be logged.
	fn logg(&self, log_level: LogLevel) {
		self.0.lock().unwrap().send(log_level).unwrap();
	}

	/// This function logs an `Info` level message.
	pub fn info(&self, message: &str) {
		self.logg(LogLevel::Info(message.to_owned()));
	}

	/// This function logs an `Warning` level message.
	pub fn warn(&self, message: &str) {
		self.logg(LogLevel::Warning(message.to_owned()));
	}

	/// This function logs an `Error` level message.
	pub fn error(&self, message: &str) {
		self.logg(LogLevel::Error(message.to_owned()));
	}

	/// This function logs a `Stop` level message.
	///
	/// This kind of message just signals the correct end of the program. If this is not at the end of your log, your program crashed.
	pub fn stop(&self) {
		self.logg(LogLevel::Stop);
	}
}


/// This function initializes the logging thread.
fn logger(receiver: Receiver<LogLevel>) {
    loop {
        match receiver.recv() {
        	Ok(data) => {
                match data {
                	LogLevel::Info(message) => info!("{}", message),
                	LogLevel::Warning(message) => warn!("{}", message),
                	LogLevel::Error(message) => error!("{}", message),
                	LogLevel::Stop => {
                		info!("The program has been properly closed.");
                		break;
                	},
                };
            }
            Err(_) => {
            	error!("The program closed itself in an unexpected way.");
            }
        }
    }
}