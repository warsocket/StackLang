use std::fs::File;
use std::env;
use std::process::exit;
use std::num::Wrapping;
use std::arch::asm;
use std::io::{Read, Write, stdin, stdout, stderr};
use std::collections::HashMap;

use regex::Regex;


const TOKENS:[u8;16] = [ //encoding 1 nibble pertoken, 2 per byte
    b'!', // HIGH ALL write new value to stack with all bits set to 1 (-1)
    b'^', // XOR
    b'|', // OR
    b'&', // AND

    b'+', // ADD
    b'-', // SUB
    b'*', // MUL (returns 2 stack numbners)
    b'/', // DIV (/0  = 0, can be used for test by performing x/x (0 when equal 1 when not equal))

    b'$', // Switch stack
    b'~', // Head stackA <-> head stackB 
    b'=', // Duplicate top value
    b'@', // SKIP aka JUMP (how much to jump extra after CP increases)

    b'?', // READ fs to stack
    b'.', // WRITE write byte to fs
    b'0', // SHL 1
    b'1', // SHL 1; | 1
];

fn main() {

    let mut args = env::args();
    args.next();

    let mut debug = false;
    let mut strict = false;

    let params:Vec<String> = args.collect();
    let mut filename = "".to_owned();

    let mut mode = Mode::RUN;
    enum Mode{
        RUN,
        COMPILE,
        BYTECODE,
        DUMP,
    }


    for p in params{
        let param = p.as_str();

        if param.starts_with("--"){
            match(param){
                "--help" => {
                    eprintln!("Usage stackofstacks [--debug, --compile, --bytecode] FILENAME");
                    eprintln!("  --debug     Shows debug / trace on STDERR while runniogn program");
                    eprintln!("  --dump      Dumps the raw (macro expanded) code");
                    eprintln!("  --strict    Aborts when popping from empty stack or accessing uninitialised ram");
                    eprintln!("  --compile   Compiles program to bytecode (emitted on STDOUT)");
                    eprintln!("  --bytecode  Runs compiled bytecode instead of text");
                },                
                "--debug" => {
                    debug = true;
                },
                "--dump" => {
                    mode = Mode::DUMP;
                },                
                "--strict" => {
                    strict = true;
                },
                "--compile" => {
                    mode = Mode::COMPILE;
                },
                "--bytecode" => {
                    mode = Mode::BYTECODE;
                },
                _ => {
                    eprintln!("Unknown option '{}' !", param);
                    exit(1);
                }
            }   
        }else{
            if filename != ""{
                eprintln!("Only one filename allowed!");
                exit(1);                    
            }
            filename = p;         
        }


    }

    if filename == ""{
        eprintln!("No filename specified!");
        exit(1);
    }

    let mut file = match File::open(filename)
    {
        Ok(f) => f,
        Err(_) => {
            eprintln!("Error opening file");
            exit(1);
        }
    };

    let size:usize = match file.metadata()
    {
        Ok(l) => l,
        Err(_) => {
            eprintln!("File has no filesize");
            exit(1);
        }
    }.len().try_into().unwrap();
    
    let mut script_bytes:Vec<u8> = Vec::with_capacity(size.try_into().unwrap());
    script_bytes.resize(size, 0);
    file.read(&mut script_bytes);

    match(mode){
        Mode::RUN => { 
            run(&tokenise(script_bytes), debug, strict); 
        },
        Mode::COMPILE => { 
            let mut out = stdout().lock();
            out.write( &compile(&tokenise(script_bytes)) ); 
        },
        Mode::BYTECODE => { 
            run(&bytecode(&script_bytes), debug, strict); 
        },
        Mode::DUMP => { 
            let pure_script = tokenise(script_bytes);

             //input is already screened to not contain non-ascii characters by the tokenise funtion
            let string = std::str::from_utf8(&pure_script).expect("INTERPRETER's FAULT: The function 'tokenise' should have checked for non ascii characters!");
            let mut index = 0;

            for token in string.chars(){
                eprintln!("0x{:#018X}:  {}", index, token);
                index += 1;
            }

        },        
    }

}
// ^(\-?[0-9]+|@)([+|\-](@([0-9]+)))*$
fn expand(input:&Vec<u8>, labels:&HashMap<Vec<u8>,usize>, index:&usize)->Vec<u8>{

    // if input.len() == 0{
    //     return vec!();
    // }

    //input is already screened to not contain non-ascii characters by the tokenise funtion
    let string = std::str::from_utf8(input).expect("INTERPRETER's FAULT: The function 'tokenise' should have checked for non ascii characters!");
    let re = Regex::new("^(?<num>\\-?[0-9]+|[a-z]+|@)((?<op>[+|\\-])(@|([a-z]+)|([0-9]+)))*$").unwrap();
    
    if re.is_match(string){ // Valid Macro

        
        let resolve_num = |num_or_symbol:&str|->i64{
            //can only be a number since we checked using regex

            let result_num:Result<i64,_> = num_or_symbol.parse(); //can only be a number since we checked using regex, so if not number its a variable to the labels map
            let num:i64;

            if let Ok(x) = result_num{ //got the number
                num = x;
            }else if num_or_symbol == "@"{
                num = *index as i64+64+1;       //<===== SET SIZE OF EXPANSION ITSELF HERE
            }else{ // more processing needed (its a label)
                if let Some(n) = labels.get(num_or_symbol.as_bytes()){
                    num = *n as i64;
                }else{
                    // Referenced label is not found
                    eprintln!("Macro parsing error: reference to undefined label: ':{}'", num_or_symbol);
                    exit(1);
                }
            }

            num
        };


        let init_capture = re.captures(string).unwrap(); //get init number
        let repeat = Regex::new("((?<op>[+|\\-])(?<num>@|([0-9]+)))").unwrap();
        let mut it = repeat.captures_iter(string);

        let mut acc:i64 = resolve_num(&init_capture["num"]);

        // The repeat part also matches on the leading numbe rif its negative, so we correct for it.
        if acc < 0 { //equivalent to:  if &init_capture["num"][0..1] == "-"
            it.next();
        }
        
        for data in it{
            let number_str:&str = &data["num"];
            let num = resolve_num(number_str);

            match &data["op"]{
                "+" => {acc += num}
                "-" => {acc -= num}
                _ => {panic!("INTERPRETER's FAULT: The operator string should only be + or - du to the receeding Regex!")}
            }
        }

        // println!("{:064b}", -1i64);
        return Vec::from(format!("!{:064b}", acc));


    }else{ //Ivalid Macro
        eprintln!("Illegal Macro: [{}]", string);
        exit(-1);
    }

}


fn tokenise(script_bytes:Vec<u8>) -> Vec<u8>{


    #[derive(Copy, Clone)]
    enum State{
        Source,
        Comment,
        Macro,
        Label,
    }

    
    let mut pure_script:Vec<u8> = vec!();
    let mut ignore_to_newline = false;

    let mut state = State::Source;
    let mut line_count = 1;
    let mut char_count = 1;

    let mut buffer:Vec<u8> = vec!();
    let mut labels:HashMap<Vec<u8>, usize> = HashMap::new();

    for token in script_bytes{


        if token >= 0x80 {
            eprintln!("Parsing error: Illegal (Non ASCII) character found at {}:{}", line_count, char_count);
            exit(1);
        }
        if token != 0x0D{ //CR not counted
            char_count += 1;
        }
        if token == 0x0A{
            char_count = 1;
            line_count += 1;
        }

        match state{
            State::Source =>{
                if TOKENS.contains(&token){
                    pure_script.push(token);
                }else{
                    match token{
                        b'#' => {
                            state = State::Comment;
                        },
                        b'[' => {
                            state = State::Macro;
                        },
                        b':' => {
                            state = State::Label;
                        },                        
                        _ => (), //preceived as comment
                    }
                }
            },
            State::Comment =>{
                match token{
                    b'\n' => {
                        state = State::Source;
                    },
                    _ => (), //preceived as comment
                }                
            },
            State::Macro =>{
                match token{
                    b']' => {
                        let expanded_macro = expand(&buffer, &labels, &pure_script.len());

                        for token in &expanded_macro{
                            if !TOKENS.contains(&token){
                                panic!("INTERPRETER's FAULT: Invalid tokens in macro output!");
                            }
                        }
                        
                        pure_script.extend(expanded_macro);

                        state = State::Source;
                        buffer = vec!();
                    },
                    b => {
                        buffer.push(b);
                    }
                }                
            }
            State::Label =>{ //97=122
                if (97 <= token) && (token <= 122){
                    buffer.push(token);
                }else{
                    labels.insert(buffer, pure_script.len());

                    state = State::Source;
                    buffer = vec!();
                }   
            }
        }


    }

    // println!("{}", std::str::from_utf8(&pure_script).unwrap());
    pure_script

}

fn compile(code:&Vec<u8>) -> Vec<u8>{
    let mut opcodes:HashMap<u8, u8> = HashMap::new();

    let mut high_bits = true;
    let mut out:Vec<u8> = vec!();

    for (index, token) in TOKENS.iter().enumerate(){
        opcodes.insert(*token, index as u8);
    }

    //Since opcode 0 (PUSH(-1)) always works and is benign on itself, we dont care is lasat nibble contains just that.
    for token in code{
        if high_bits{
            out.push( *opcodes.get(token).unwrap() << 4 );
        }else{
            let i:usize = out.len()-1;
            out[i] |= *opcodes.get(token).unwrap();
        }
        high_bits = !high_bits;
    }

    out
}


fn bytecode(bytecode:&Vec<u8>) -> Vec<u8>{
    let mut code:Vec<u8> = vec!();

    for byte in bytecode{
        code.push(TOKENS[usize::from(byte>>4)]);
        code.push(TOKENS[usize::from(byte&0b1111)]);
    }

    code
}


trait Oos{
    fn oos(self:&Self, infinite_stack:&bool)->i64;
}

impl Oos for Option<i64>{
    fn oos(self:&Self, strict:&bool) -> i64{
        match(self){

            Some(v) => *v,
            None => {
                if *strict{
                    eprintln!("Strict mode violation: Stack depleted");
                    exit(1);
                }else{
                    -1
                }
            }

        }

    }    
}

fn run(code:&Vec<u8>, debug:bool, strict:bool){

    if code.len() == 0{return}

    let mut index = 0;
    let mut ram:HashMap<i64, i64> = HashMap::new();

    let mut stacks: [Vec<i64>;2] = [vec!(),vec!()];
    let mut stack: &mut Vec<i64>;
    let mut stack_index = 0;
    stack = &mut stacks[stack_index];

    loop{

        if debug{
            eprintln!("{:?}", &stack);
            stack = &mut stacks[!stack_index&1];
            eprintln!("{:?}", &stack);
            stack = &mut stacks[stack_index&1];
            eprintln!("{:#018X}: {}", index, code[index] as char);
        }

        match(code[index]){
            b'$' => {//Switch stack
                stack_index = !stack_index&1;
                stack = &mut stacks[stack_index];
            }
            b'~' => {//Xchange stack heads
                let a = stack.pop().oos(&strict);
                stack = &mut stacks[!stack_index&1];
                let b = stack.pop().oos(&strict);
                
                stack.push(a);
                stack = &mut stacks[stack_index];
                stack.push(b);
            }
            b'=' => {// Duplicate top value
                let a = stack.pop().oos(&strict);
                stack.push(a);
                stack.push(a);
            }            
            b'+' => {
                let b = stack.pop().oos(&strict);
                let a = stack.pop().oos(&strict);
                stack.push( (Wrapping(a)+Wrapping(b)).0 );
            }
            b'-' => {
                let b = stack.pop().oos(&strict);
                let a = stack.pop().oos(&strict);
                stack.push( (Wrapping(a)-Wrapping(b)).0 );
            }
            b'*' => { // MULTIPLY stack:[a,b] -> [low, high]
                let b = stack.pop().oos(&strict);
                let a = stack.pop().oos(&strict);
                let x = i128::from(a)*i128::from(b);
                stack.push(x as i64);
                // stack.push((x >> 64) as i64); //just lose the eccess
            }
            b'/' => {
                let b = stack.pop().oos(&strict);
                let a = stack.pop().oos(&strict);
                if b != 0{
                    stack.push(a/b);
                }else{
                    stack.push(0); //div by 0 is 0 by design (can replace test)
                }
            }
            b'|' => {
                let b = stack.pop().oos(&strict);
                let a = stack.pop().oos(&strict);
                stack.push( a | b );
            }
            b'&' => {
                let b = stack.pop().oos(&strict);
                let a = stack.pop().oos(&strict);
                stack.push( a & b );
            }
            b'^' => {
                let b = stack.pop().oos(&strict);
                let a = stack.pop().oos(&strict);
                stack.push( a ^ b );
            }
            b'!' => {
                stack.push( -1 );
            }
            b'1' => {
                let a = stack.pop().oos(&strict);
                stack.push((a << 1) | 0b1);
            }
            b'0' => {
                let a = stack.pop().oos(&strict);
                stack.push((a << 1));
            }
            b'@' => {
                let a = stack.pop().oos(&strict);
                if a == -1 {break} // This would lead to perpetual spinlock basically haling ,execution, so better make it an exit strategy
                index = (Wrapping(index)+Wrapping(a as usize)).0;
            }
            b'?' => {
                let mut buffer:[u8;1] = [0];
                let stack_value = match stdin().lock().read_exact(&mut buffer){
                    Ok(_) => buffer[0] as i64,
                    Err(_) => -1,   
                };
                stack.push(stack_value);
            }
            b'.' => {
                let ch = stack.pop().oos(&strict);

                let buffer:[u8;1] = [ch as u8];
                let _ = stdout().lock().write(&buffer);
            }            
            _ => {
                panic!("INTERPRETER's FAULT: Invalid token!")
            }
        }

        index = (Wrapping(index)+Wrapping(1usize)).0;
        if (index<0) | (index >= code.len()) {
            if strict{
                eprintln!("Strict mode violation: Outside of code memmory (offset: 0x{:#018X})", index);
                exit(1);
            }else{

            }   index = 0;
        }
    }

    if debug{
        eprintln!("{:?}", &stack);
        stack = &mut stacks[!stack_index&1];
        eprintln!("{:?}", &stack);
    }
    

}
