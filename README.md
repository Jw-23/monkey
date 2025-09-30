# Monkey 解释器

一个用 Rust 编写的 **Monkey** 编程语言解释器，支持 REPL 交互模式和文件执行模式。

## 特性

- **整型与浮点数支持**：支持大整数运算和浮点数计算
- **字符串支持**：支持字符串字面量和字符串拼接
- **布尔值与控制流**：支持 `if/else` 条件表达式
- **函数与闭包**：支持高阶函数和闭包
- **运算符**：支持算术、比较和逻辑运算符
- **交互式 REPL**：启动 REPL 实时编写和执行代码
- **AST 可视化**：支持显示抽象语法树

## 安装

确保已安装 Rust 工具链（Rust 1.75+）：

```bash
cargo build --release
```

可执行文件位于 `target/release/monkey`。

## 使用方式

### 交互式 REPL

直接运行启动交互式解释器：

```bash
cargo run
# 或
./target/release/monkey repl
```

### 运行源文件

```bash
cargo run -- run example.mk
```

### 显示 AST

```bash
cargo run -- run example.mk --ast
```

## 语言语法

### 变量声明

```monkey
let x = 5;
let name = "Monkey";
let pi = 3.14;
```

### 算术运算

```monkey
let a = 10;
let b = 20;
a + b        // 30
a * b        // 200
(5 + 10) * 2 // 30
```

### 布尔运算

```monkey
true
false
!true        // false
1 < 2        // true
3 == 3       // true
"hello" != "world"  // true
```

### 条件表达式

```monkey
if (x > 10) {
    "x is greater than 10"
} else {
    "x is not greater than 10"
}
```

### 函数

```monkey
let add = fn(a, b) {
    a + b
};
add(5, 10)   // 15

// 闭包
let new_adder = fn(x) {
    fn(y) { x + y }
};
let add_two = new_adder(2);
add_two(3)   // 5
```

### 字符串拼接

```monkey
"Hello" + " " + "World!"  // "Hello World!"
```

## 项目结构

```
src/
├── main.rs        # 命令行入口
├── lib.rs         # 核心库定义
├── compiler.rs    # 编译器/解析器入口
├── tokenizer/     # 词法分析器
│   ├── mod.rs
│   └── token.rs   # Token 定义
├── syntax/        # 语法分析器
│   └── mod.rs
├── eval/          # 求值器
│   └── mod.rs
└── logic/         # 逻辑模块
    └── mod.rs
```

## 技术栈

- **Rust** - 系统编程语言
- **clap** - 命令行参数解析
- **num-bigint** - 大整数运算
- **thiserror** - 错误处理

## 许可证

MIT
