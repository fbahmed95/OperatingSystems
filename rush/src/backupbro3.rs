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

pub struct SingleCommand{
    pub command: String,
    pub inputFile: String,
    pub outputFile: String,
    pub isBackground: bool,
    pub arguments: Vec<String>,
    pub commandOutput: String,
    // add field to save reference to potential background job
}

fn main() {
   print!("$ ");
   io::stdout().flush();
   let stdin = io::stdin();
   let mut commandHistory = Vec::new();

   for input in stdin.lock().lines() {
       let inp = input.unwrap();
       let mut currentCommand = SingleCommand {
            command: String::from(""),
            inputFile: String::from(""),
            outputFile: String::from(""),
            isBackground: false,
            arguments: Vec::new(),
            commandOutput: String::from(""),
       };
       getSingleCommand(&mut currentCommand, inp);
       commandHistory.push(&mut currentCommand);
       executeSingleCommand(&mut currentCommand, &mut commandHistory);
       print!("$ ");
       io::stdout().flush();
    }
}

fn getSingleCommand(currentCommand : &mut SingleCommand, rawCommandText: String) {
    let mut rawCommandSplit: Vec<&str> = rawCommandText.split(" ").collect();
    let mut inputFile = "";
    let mut outputFile = "";
    let command = rawCommandSplit[0].trim();
    let mut skipArg = false;

    for i in 1..rawCommandSplit.len() {
        if rawCommandSplit[i] == "&" {
            currentCommand.isBackground = true;
        } else if rawCommandSplit[i] == "<"  {
            inputFile = rawCommandSplit[i+1].trim();
            skipArg = true;
        } else if rawCommandSplit[i] == ">" {
            outputFile = rawCommandSplit[i+1].trim();
            skipArg = true;
        } else if skipArg != true {
            currentCommand.arguments.push(String::from(rawCommandSplit[i].trim()));
        } else if skipArg == true {
            skipArg = false;
        }
    }

    currentCommand.command = String::from(command);
    currentCommand.inputFile = String::from(command);
    currentCommand.outputFile = String::from(command);

    ()
}

fn executeSingleCommand(currentCommand : &mut SingleCommand, historyInput: &mut Vec<&mut SingleCommand>) {
    if currentCommand.command == "exit" {
        exitShell();
    } else if currentCommand.command == "pwd" {
        //pwdShell(&mut commandCopy);
    } else if currentCommand.command == "cd" {
        //cdShell(&mut commandCopy);
    } else if currentCommand.command == "kill" {

    } else if currentCommand.command == "history" {

    } else if currentCommand.command == "jobs" {

    } else {
        // externals
    }

    if currentCommand.outputFile.len() > 0 {
        let outputFilePath = Path::new(currentCommand.outputFile.as_str());
        let outputFilePathDisplay = outputFilePath.display();
        let mut file = match File::create(&outputFilePath) {
            Err(why) => panic!("couldn't create {}", outputFilePathDisplay),
            Ok(file) => file,
        };
        file.write_all(currentCommand.commandOutput.as_bytes());
    } else if currentCommand.commandOutput.len() > 0 {
        println!("{}", currentCommand.commandOutput);
    }
}

fn exitShell(){
    process::exit(0);
}

/*fn historyShell(mut historyInput: Vec<String>){
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
