#[allow(non_snake_case)]
#[allow(dead_code)]
#[allow(unused_mut)]
#[allow(unused_assignments)]
extern crate libc;
use std::io::{self, Write};
use std::process;
use std::process::Stdio;
use std::env;
use std::path::Path;
use std::process::Command;
use std::io::stdout;
use std::io::prelude::*;
use std::borrow::Cow;
use std::fs::File;
use std::process::Child;
use std::cell::RefCell;
use std::os::unix::io::FromRawFd;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::IntoRawFd;
#[derive(Debug)]

pub struct SingleCommand{
    pub rawCommand: String,
    pub isBackground: bool,
    pub commandOutput: String,
    pub processes: Vec<Option<RefCell<Child>>>,
}

fn main() {
   print!("$ ");
   io::stdout().flush();
   let stdin = io::stdin();
   let mut commandHistory = Vec::new();

   for input in stdin.lock().lines() {
       let inp = input.unwrap();
       let mut currentCommand = SingleCommand {
            rawCommand: inp.clone(),
            isBackground: false,
            commandOutput: String::from(""),
            processes: Vec::new(),
       };
       runSingleCommand(&mut currentCommand, inp, &mut commandHistory);
       commandHistory.push(currentCommand);
       print!("$ ");
       io::stdout().flush();
    }
}

fn runSingleCommand(currentCommand : &mut SingleCommand, rawCommandText: String, historyInput: &mut Vec<SingleCommand>) {
    let mut pipeCommandSplit : Vec<&str> = rawCommandText.split(" | ").collect();

    let mut inputFile = "";
    let mut outputFile = "";
    let numCommands = pipeCommandSplit.len();
    let mut isBuiltInCommand : Vec<bool> = Vec::new();

    // extract background, input and output file information
    for i in 0..numCommands {
        if pipeCommandSplit[i].find("&").is_some() {
            currentCommand.isBackground = true;
            let backgroundSplit : Vec<&str> = pipeCommandSplit[i].split("&").collect();
            let firstArg = backgroundSplit[0].trim();
            pipeCommandSplit[i] = firstArg;
        } else if pipeCommandSplit[i].find("<").is_some() {
            let inputSplit : Vec<&str> = pipeCommandSplit[i].split("<").collect();
            inputFile = inputSplit[1].trim();
            let firstArg = inputSplit[0].trim();
            pipeCommandSplit[i] = firstArg;
        } else if pipeCommandSplit[i].find(">").is_some() {
            let outputSplit : Vec<&str> = pipeCommandSplit[i].split(">").collect();
            outputFile = outputSplit[1].trim();
            let firstArg = outputSplit[0].trim();
            pipeCommandSplit[i] = firstArg;
        }
    }

    for i in 0..numCommands {
        let mut commandSplit : Vec<&str> = pipeCommandSplit[i].split(" ").map(|x| x.trim()).collect();
        let command = commandSplit[0];
        if command == "exit" {
            exitShell();
        } else if command == "pwd" {
            pwdShell(currentCommand);
            isBuiltInCommand.push(true);
            currentCommand.processes.push(None);
        } else if command == "kill" {
            killShell(currentCommand, commandSplit[1]);
            isBuiltInCommand.push(true);
            currentCommand.processes.push(None);
        } else if command == "cd" {
            cdShell(currentCommand, commandSplit[1]);
            isBuiltInCommand.push(true);
            currentCommand.processes.push(None);
        } else if command == "history" {
            historyShell(currentCommand, historyInput);
            isBuiltInCommand.push(true);
            currentCommand.processes.push(None);
        } else if command == "jobs" {
            jobsShell(currentCommand, historyInput);
            isBuiltInCommand.push(true);
            currentCommand.processes.push(None);
        } else {
            let mut externalCommand = Command::new(command);
            let isFirstCommand = i == 0;
            let isLastCommand = i == numCommands - 1;

            if commandSplit.len() > 1 {
                let mut commandArgs = commandSplit.clone();
                commandArgs.remove(0);
                for x in 0..commandArgs.len() {
                    externalCommand.arg(commandArgs[x]);
                }
            }

            // if not last, pipe stdout
            if !isLastCommand {
                externalCommand.stdout(Stdio::piped());
            } else if isLastCommand && outputFile.len() > 0 {
                // if last and we have an outputfile
                let fileResult = File::create(Path::new(outputFile));
                if (fileResult.is_ok()) {
                    externalCommand.stdout(unsafe {
                        Stdio::from_raw_fd(fileResult.ok().unwrap().into_raw_fd())
                    });
                }
            }

            // if not first and previous was not built in, link pipes
            if !isFirstCommand && !isBuiltInCommand[i-1] {
                let prevProcessOptional = &currentCommand.processes[i-1];
                let mut prevProcessRefCell = prevProcessOptional.as_ref();
                let mut prevProcess = prevProcessRefCell.unwrap().borrow_mut();
                externalCommand.stdin(unsafe {
                    Stdio::from_raw_fd(prevProcess.stdout.as_mut().unwrap().as_raw_fd())
                });
            } else if !isFirstCommand {
                externalCommand.stdin(Stdio::piped());
            } else if isFirstCommand && inputFile.len() > 0 {
                // if first and we have an inputfile
                let fileResult = File::open(Path::new(inputFile));
                if (fileResult.is_ok()) {
                    externalCommand.stdin(unsafe {
                        Stdio::from_raw_fd(fileResult.ok().unwrap().into_raw_fd())
                    });
                }
            }

            let mut child = externalCommand.spawn().unwrap();

            // if not first and the previous was a built in command
            if !isFirstCommand && isBuiltInCommand[i-1] {
                child.stdin.as_mut().unwrap().write_all(currentCommand.commandOutput.as_bytes());
            }

            currentCommand.processes.push(Some(RefCell::new(child)));
            isBuiltInCommand.push(false);
            currentCommand.commandOutput = String::from("");
        }
    }

    if !currentCommand.isBackground {
        // if not running in background, wait for completion of any child processes
        waitForProcesses(currentCommand);
    }

    if currentCommand.commandOutput.len() > 0 {
        print!("{}", currentCommand.commandOutput);
    }
}

fn exitShell(){
    process::exit(0);
}

fn pwdShell(command: &mut SingleCommand) {
    let path = env::current_dir().unwrap();
    let mut pathString = path.to_str().unwrap();
    let mut output = String::from(pathString.to_string());
    output.push('\n');
    command.commandOutput = output;
}

fn killShell(command: &mut SingleCommand, pid: &str){
    unsafe {
        libc::kill(i32::from_str_radix(pid, 10).unwrap(), 15);
    }
    command.commandOutput = String::from("");
}

fn cdShell(command: &mut SingleCommand, path: &str){
    let root = Path::new(path);
    assert!(env::set_current_dir(&root).is_ok());
    command.commandOutput = String::from("");
}

fn historyShell(command: &mut SingleCommand, historyInput: &mut Vec<SingleCommand>){
    let mut output = String::from("");
    let len = historyInput.len();
    for x in 1..historyInput.len()+1 {
        output.push_str(&format!("{:5}  {}", x, historyInput[x-1].rawCommand.trim()));
        output.push('\n');
    }
    command.commandOutput = output;
}

fn jobsShell(command: &mut SingleCommand, historyInput: &mut Vec<SingleCommand>){
    let mut output = String::from("");
    let len = historyInput.len();
    for x in 0..len{
        let currentCommand = &historyInput[x];
        for i in 0..currentCommand.processes.len() {
            let processOptional = currentCommand.processes[i].as_ref();
            if (processOptional.is_some()) {
                let mut child = processOptional.unwrap().borrow_mut();
                let tryWait = child.try_wait();
                match tryWait {
                    Ok(Some(status)) => {},
                    _ => {
                        let splitBackground : Vec<&str> = currentCommand.rawCommand.split("&").map(|x| x.trim()).collect();
                        output.push_str(splitBackground[0]);
                        output.push_str("\n");
                    }
                }
            }
        }
    }
    command.commandOutput = output;
}

fn waitForProcesses(currentCommand: &mut SingleCommand) {
    for i in 0..currentCommand.processes.len() {
        let processOptional = currentCommand.processes[i].as_ref();
        if (processOptional.is_some()) {
            let mut child = processOptional.unwrap().borrow_mut();
            child.wait();
        }
    }
}
