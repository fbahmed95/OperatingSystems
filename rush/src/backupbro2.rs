#[allow(non_snake_case)]
#[allow(dead_code)]
#[allow(unused_mut)]
#[allow(unused_assignments)]
extern crate libc;
use std::io::{self, Write};
use std::process;
use std::env;
use std::path::Path;
use std::process::Command;
use std::io::stdout;
use std::io::prelude::*;
use std::borrow::Cow;
use std::fs::File;
#[derive(Debug)]

pub struct singleCommand<'a>{
    pub command: &'a str,
    pub inputFile: &'a str,
    pub outputFile: &'a str,
    pub isBackground: bool,
    pub arguments: Vec<&'a str>,
    pub commandOutput: String,
    // add field to save reference to potential background job
}

fn main() {
   print!("$ ");
   io::stdout().flush();
   let stdin = io::stdin();
   let mut historyInput = Vec::new();

   for input in stdin.lock().lines() {
       match input {
           Ok(_) => (),
           Err(_) => break,
       }
       let inp = input.unwrap();
       let command = getSingleCommand(&inp);
       historyInput.push(command);
       //executeSingleCommand(historyInput);
       print!("$ ");
       io::stdout().flush();
    }
}

fn getSingleCommand(rawCommandText: &str) -> (singleCommand, &str)  {
    let mut rawCommandSplit: Vec<&str> = rawCommandText.split(" ").collect();
    let mut arguments = Vec::new();
    let mut isBackground = false;
    let mut inputFile = "";
    let mut outputFile = "";
  //  let mut outputCommand = "";
    let command = rawCommandSplit[0].trim();
    let mut skipArg = false;

    for i in 1..rawCommandSplit.len() {
        if rawCommandSplit[i] == "&" {
            isBackground = true;
        } else if rawCommandSplit[i] == "<"  {
            inputFile = rawCommandSplit[i+1].trim();
            skipArg = true;
        } else if rawCommandSplit[i] == ">" {
            outputFile = rawCommandSplit[i+1].trim();
            skipArg = true;
        } else if skipArg != true {
            arguments.push(rawCommandSplit[i].trim());
        } else if skipArg == true {
            skipArg = false;
        }
    }

    return (singleCommand {
        command: command,
        isBackground: isBackground,
        arguments: arguments,
        inputFile: inputFile,
        outputFile: outputFile,
        commandOutput: String::from(""),
    };
}

/*fn executeSingleCommand(mut historyInput: Vec<singleCommand>) {
    let mut commandCopy = historyInput.last().unwrap();

    if commandCopy.command == "exit" {
        exitShell();
    } else if commandCopy.command == "pwd" {
        pwdShell(&mut commandCopy);
    } else if commandCopy.command == "cd" {
        cdShell(&mut commandCopy);
    } else if commandCopy.command == "kill" {

    } else if commandCopy.command == "history" {

    } else if commandCopy.command == "jobs" {

    } else {
        // externals
    }

    if commandCopy.outputFile.len() > 0 {
        let outputFilePath = Path::new(commandCopy.outputFile);
        let outputFilePathDisplay = outputFilePath.display();
        let mut file = match File::create(&outputFilePath) {
            Err(why) => panic!("couldn't create {}", outputFilePathDisplay),
            Ok(file) => file,
        };
        file.write_all(commandCopy.commandOutput.as_bytes());
    } else if commandCopy.commandOutput.len() > 0 {
        println!("{}", commandCopy.commandOutput);
    }
}

fn exitShell(){
    process::exit(0);
}

fn historyShell(mut historyInput: Vec<String>){
    let len = historyInput.len();
    for x in 0..len{
        println!("  {}  {}", (x+1) , historyInput[x]);
    }
}

fn pwdShell(command: &mut singleCommand) {
    let path = env::current_dir().unwrap();
    let mut pathString = path.to_str().unwrap();
    // todo: remove testing
    //println!("{}", pathString);
    command.commandOutput = String::from(pathString.to_string());
}

fn cdShell(command: &mut singleCommand){
    if command.inputFile.len() > 0 {
        let fileContents = getFileContents(command.inputFile);
        let root = Path::new(fileContents.as_str().trim());
        assert!(env::set_current_dir(&root).is_ok());
    } else {
        let root = Path::new(command.arguments[0]);
        assert!(env::set_current_dir(&root).is_ok());
    }

    
}

fn getFileContents(filepath: &str) -> String {
    let path = Path::new(filepath);
    let display = path.display();
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}", display),
        Ok(file) => file,
    };

    let mut s = String::new();
    file.read_to_string(&mut s);
    return s;
}


fn killShell(shellargs: &str){
    unsafe{
        libc::kill(i32::from_str_radix(shellargs,10).unwrap(),15);
    }
}

fn backgroundCommandShell(singleCommand: &str, argument: &str){
    println!("{}, {}", singleCommand, argument);
    let mut bgCommand = Command::new(singleCommand).arg(argument).spawn().expect("failed");
}

fn externals(singleCommand: &Vec<&str>){
     if singleCommand.len() == 1{
        let mut ans = Command::new(singleCommand[0].trim()).spawn().expect("ls command failed to start");
        ans.wait();
    } else if singleCommand.len() == 2 {
        let mut ans = Command::new(singleCommand[0].trim()).arg(singleCommand[1].trim()).spawn().unwrap();
        ans.wait().unwrap();
    } else if singleCommand.len() == 3 {
        let mut ans = Command::new(singleCommand[0].trim()).arg(singleCommand[1]).arg(singleCommand[2]).spawn().expect("command failed to start");
        ans.wait();
    }
}*/
