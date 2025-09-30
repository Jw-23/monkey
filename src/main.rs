
use monkey::compile;
use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Parser)]
#[command(name = "monkey")]
#[command(about = "Monkey 编程语言解释器", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 运行 Monkey 源代码文件
    Run {
        /// 要执行的源代码文件路径
        #[arg(value_name = "FILE")]
        file: PathBuf,
        
        /// 显示抽象语法树 (AST)
        #[arg(short, long)]
        ast: bool,
    },
    
    /// 启动交互式 REPL (Read-Eval-Print Loop)
    Repl,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Run { file, ast }) => {
            if let Err(e) = run_file(file, *ast) {
                eprintln!("错误: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Repl) | None => {
            if let Err(e) = run_repl() {
                eprintln!("REPL 错误: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn run_file(path: &PathBuf, show_ast: bool) -> Result<(), Box<dyn std::error::Error>> {
    let source = fs::read_to_string(path)?;
    
    // 编译源代码
    let program = compile(&source).map_err(|e| format!("编译失败: {}", e))?;
    
    if show_ast {
        println!("=== 抽象语法树 (AST) ===");
        println!("{:#?}", program);
        println!();
    }
    
    // 创建求值器和环境
    let evaluator = monkey::eval::Evaluator::new(source.clone());
    let env = Rc::new(RefCell::new(monkey::eval::Environment::new()));
    
    // 执行程序
    let result = evaluator.eval_program(&program, env)
        .map_err(|e| format!("执行失败: {}", e))?;
    
    // 如果结果不是 Null，则打印结果
    if !matches!(result, monkey::eval::Object::Null) {
        println!("{}", result);
    }
    
    Ok(())
}

fn run_repl() -> Result<(), Box<dyn std::error::Error>> {
    println!("Monkey 编程语言 v0.1.0");
    println!("输入 '.exit' 退出, '.help' 查看帮助");
    println!();
    
    let env = Rc::new(RefCell::new(monkey::eval::Environment::new()));
    
    loop {
        print!(">> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let input = input.trim();
        
        // 处理特殊命令
        match input {
            ".exit" | ".quit" => {
                println!("再见！");
                break;
            }
            ".help" => {
                print_help();
                continue;
            }
            "" => continue,
            _ => {}
        }
        
        // 编译并执行输入
        match compile(input) {
            Ok(program) => {
                let evaluator = monkey::eval::Evaluator::new(input.to_string());
                match evaluator.eval_program(&program, Rc::clone(&env)) {
                    Ok(result) => {
                        if !matches!(result, monkey::eval::Object::Null) {
                            println!("{}", result);
                        }
                    }
                    Err(e) => eprintln!("执行错误: {}", e),
                }
            }
            Err(e) => eprintln!("编译错误: {}", e),
        }
    }
    
    Ok(())
}

fn print_help() {
    println!("Monkey REPL 命令:");
    println!("  .exit, .quit  - 退出 REPL");
    println!("  .help         - 显示此帮助信息");
    println!();
    println!("示例代码:");
    println!("  let x = 5;");
    println!("  let y = 10;");
    println!("  x + y");
    println!("  let add = fn(a, b) {{ a + b }};");
    println!("  add(x, y)");
    println!();
}
