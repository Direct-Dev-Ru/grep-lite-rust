use std::process;
use clap::Parser;
// use std::env;
use regex::Regex;
use std::fs;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

/// Basic example of a CLI tool using clap
#[derive(Parser, Debug)]
#[command(name = "grep_lite", version = "1.0", about = "A simple lite grep tool")]
struct Cli {
    /// Enable verbose mode option
    #[arg(short, long)]
    verbose: bool,

    /// regular option
    #[arg(short, long)]
    regular: bool,

    /// pattern to search
    #[arg(short, long)]
    pattern: String,

    /// input text to search
    #[arg(short, long)]
    input: Option<String>,

    /// output type: text or json
    #[arg(short, long)]
    output_type: Option<String>,

    /// context lines count
    #[arg(short, long)]
    ctx_count: Option<i32>,
}

fn read_file_lines_as_strs(input: &str) -> io::Result<Vec<String>> {
    // Open the file
    let file = File::open(input)?;
    let reader = BufReader::new(file);

    // Collect lines into a vector of Strings
    let lines: Vec<String> = reader
        .lines() // Iterate over lines
        .collect::<Result<_, _>>()?; // Handle potential IO errors during reading

    Ok(lines)
}

fn main() {
    _ = read_file_lines_as_strs;

    let clap_args = Cli::parse();
    let search_term = clap_args.pattern;
    let re = Regex::new(&search_term).unwrap();
    let file_path_or_text = clap_args.input;
    let ctx_count = match clap_args.ctx_count {
        Some(count) => count,
        None => 0,
    };
    let output_type = match &clap_args.output_type {
        // Клонируем значение, если оно допустимо
        Some(o_type) if o_type == "text" || o_type == "json" => o_type.clone(),
        _ => "text".to_string(), // Используем "text" по умолчанию
    };
    _ = output_type.to_string();

    // Read the file and handle errors
    // let example_lines = match read_file_lines_as_strs("test.txt") {
    //     Ok(lines) => lines,
    //     Err(e) => {
    //         eprintln!("Error reading file: {}", e);
    //         return;
    //     }
    // };
    

    let mut input_text = if atty::is(atty::Stream::Stdin) {
        // Если stdin не используется (нет pipe), читаем из файла
        match file_path_or_text {
            Some(ref path) => {
                match fs::read_to_string(path) {
                    Ok(content) => content,
                    Err(_) => "error_reading_file".to_string(), // Если файл не найден, считаем все это текстом и возвращаем его
                }
            }
            None => {
                eprintln!(
                    "Необходимо указать путь к файлу или текст или передать текст через pipe."
                );
                return;
            }
        }
    } else {
        // Если stdin используется (есть pipe), читаем из него
        let stdin = io::stdin();
        let mut lines = stdin.lock().lines(); // Читаем строки из stdin
        let mut stdin_input_text = String::new();
        while let Some(line) = lines.next() {
            match line {
                Ok(line_content) => {
                    stdin_input_text.push_str(&line_content);
                    stdin_input_text.push('\n'); // Добавляем перенос строки
                }
                Err(e) => {
                    eprintln!("Ошибка при чтении из stdin: {}", e);
                    return;
                }
            }
        }
        stdin_input_text
    };

    if input_text == "error_reading_file" {
        // process::exit(1); // Exit with non-zero code for failure
        input_text = match file_path_or_text {
            Some(content) => content,
            None => "".to_string(),
        };
    }

    let mut tags: Vec<u32> = vec![];
    let mut ctx: Vec<Vec<(i32, String)>> = vec![];
    let ctx_lines = ctx_count as usize;
    for (i, line) in input_text.lines().enumerate() {
        let is_matching;
        if clap_args.regular {
            let contains_substr = re.find(line);
            // println!("{:?}", contains_substr);
            is_matching = match contains_substr {
                Some(_) => true,
                None => false,
            }
        } else {
            is_matching = line.contains(&search_term);
        }
        if is_matching {
            // println!("{} : {}", i, line);
            tags.push(i as u32);
            // Создаем контекст для текущей строки
            let mut v = Vec::new();
            for j in (i.saturating_sub(ctx_lines))..=(i + ctx_lines) {
                if let Some(context_line) = input_text.lines().nth(j) {
                    let context_index: i32 = j as i32;
                    let context_line_str = String::from(context_line);
                    v.push((context_index, context_line_str));
                }
            }
            ctx.push(v);
        }
    }

    println!("{}", "-".repeat(50));

    for (i, _vec) in ctx.iter().enumerate() {
        let middle_index = _vec.len() / 2; // Индекс средней строки

        for (_j, line) in _vec.iter().enumerate() {
            if _j == middle_index {
                println!("Match_{}. line_{}. > {}", i + 1, line.0, line.1);
            } else {
                let diff = ((middle_index as i32) - (_j as i32)).abs();
                // println!("{}#{}#{}",diff,_j,middle_index);
                println!(
                    "Match_{}. line_{}. {} {}",
                    i + 1,
                    line.0,
                    ">>".repeat(diff as usize),
                    line.1
                );
            }
        }
        // Разделитель между группами совпадений
        println!("{}", "-".repeat(50));
    }

    process::exit(0); // Exit with 0 for success
}
