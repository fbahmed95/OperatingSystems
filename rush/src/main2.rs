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
#[derive(Debug)]

pub struct singleCommand<'a>{
    pub command: &'a str,
    pub inputFile: &'a str,
    pub outputFile: &'a str,
    pub isBackground: bool,
    pub arguments: Vec<&'a str>,
    // add field to save reference to potential background job
}

fn main() {
   print!("$ ");
   let mut input = String::new();
   io::stdout().flush();
   let stdin = io::stdin();
   let mut historyInput = Vec::new();
   let mut backgroundJobs = Vec::new();

   for input in stdin.lock().lines() {
       match input {
           Ok(_) => (),
           Err(_) => break,
       }
       let inp = input.unwrap();
       let inp1 = inp.clone();
       historyInput.push(inp);
       let command = getSingleCommand(&inp1);

       executeSingleCommand(&command, historyInput.clone());
       print!("$ ");
       io::stdout().flush();
    }
}


//
fn getSingleCommand(rawCommandText: &str) -> singleCommand  {
    let mut rawCommandSplit: Vec<&str> = rawCommandText.split(" ").collect();
    let mut arguments = Vec::new();
    let mut isBackground = false;
    let mut inputFile = "";
    let mut outputFile = "";
    let command = rawCommandSplit[0].trim();
    let mut skipArg = false;

    for i in 1..rawCommandSplit.len() {
        if (rawCommandSplit[i] == "&") {
            isBackground = true;
        } else if (rawCommandSplit[i] == "<" ) {
            inputFile = rawCommandSplit[i+1].trim();
            skipArg = true;
        } else if (rawCommandSplit[i] == ">") {
            outputFile = rawCommandSplit[i+1].trim();
            skipArg = true;
        } else if (skipArg != true){
            arguments.push(rawCommandSplit[i].trim());
        } else if (skipArg == true) {
            skipArg = false;
        }
    }

    return singleCommand {
        command: command,
        isBackground: isBackground,
        arguments: arguments,
        inputFile: inputFile,
        outputFile: outputFile,
    };
}

fn executeSingleCommand(command: &singleCommand, historyInput: Vec<&str>) {
    let mut commandOutput = "";

    if (command.command == "exit") {
        exitShell();
    } if (command.command == "pwd") {
        commandOutput = pwdShell();
    } if (command.command == "history") {
        historyShell(historyInput);
    }else {
        println!("Unsupported command");
    }

    if (command.outputFile) {
        // save commandOutput to file
    } else {
        println!("{}", commandOutput);
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

fn pwdShell() -> &str {
    let path = env::current_dir().unwrap();
    return path.display();
}

fn cdShell(shellargs: &str){
 let root = Path::new(shellargs);
assert!(env::set_current_dir(&root).is_ok());
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
    }else if singleCommand.len() == 2 {
        let mut ans = Command::new(singleCommand[0].trim()).arg(singleCommand[1].trim()).spawn().unwrap();
        ans.wait().unwrap();
    }else if singleCommand.len() == 3 {
        let mut ans = Command::new(singleCommand[0].trim()).arg(singleCommand[1]).arg(singleCommand[2]).spawn().expect("command failed to start");
        ans.wait();
    }
}
