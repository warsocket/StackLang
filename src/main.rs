use std::fs::File;
use std::env;
use std::process::exit;
use std::num::Wrapping;
use std::arch::asm;
use std::io::{Read, Write, stdin, stdout, stderr};
use std::collections::HashMap;

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
]; //Jumping outsid eof the array of instruction = HALT

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
    }


    // println!("{}", fil)

    for p in params{
        let param = p.as_str();

        if param.starts_with("--"){
            match(param){
                "--help" => {
                    eprintln!("Usage stackofstacks [--debug, --compile, --bytecode] FILENAME");
                    eprintln!("  --debug     Shows debug / trace on STDERR while runniogn program");
                    eprintln!("  --strict    Aborts when popping from empty stack or accessing uninitialised ram");
                    eprintln!("  --compile   Compiles program to bytecode (emitted on STDOUT)");
                    eprintln!("  --bytecode  Runs compiled bytecode instead of text");
                },                
                "--debug" => {
                    debug = true;
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
    }

}

fn tokenise(script_bytes:Vec<u8>) -> Vec<u8>{

    let mut pure_script:Vec<u8> = vec!();
    let mut ignore_to_newline = false;

    for token in script_bytes{

        if ignore_to_newline{
            if token == b'\n'{ ignore_to_newline = false; }
        }else{
            if TOKENS.contains(&token){
                pure_script.push(token)
            }else if token == b'#'{
                ignore_to_newline = true;
            }
        }
    }

    pure_script

}

fn compile(code:&Vec<u8>) -> Vec<u8>{
    let mut opcodes:HashMap<u8, u8> = HashMap::new();

    let mut high_bits = true;
    let mut out:Vec<u8> = vec!();

    for (index, token) in TOKENS.iter().enumerate(){
        opcodes.insert(*token, index as u8);
    }

    //Since opcode 0 (PUSH(reg)) always works and is benign on itself, we dont care is lasat nibble contains just that.
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


trait Oom{
    fn oom(self:&Self, strict:&bool)->i64;
}

impl Oom for Option<&i64>{
    fn oom(self:&Self, strict:&bool) -> i64{
        match(self){

            Some(v) => **v,
            None => {
                if *strict{
                    eprintln!("Strict mode violation: Acessing uninitialised ram");
                    exit(1);
                }else{
                    0
                }
            }

        }

    }    
}



fn run(code:&Vec<u8>, debug:bool, strict:bool){

    if code.len() == 0{return}

    let mut index = 0;
    let mut ram:HashMap<i64, i64> = HashMap::new();

    // let func_matrix:[&dyn Fn(i64) -> i64;3] = [
    //     &|a| {
    //     let mut buffer:[u8;1] = [0];
    //     stdin().lock().read_exact(&mut buffer);
    //     buffer[0] as i64
    //     }, //STDIN READ BYTE
    //     &|a| {
    //         let buffer:[u8;1] = [a as u8];
    //         stdout().lock().write(&buffer).expect("Write Error") as i64;
    //         1i64 //
    //     }, //STDOUT WRITE BYTE
    //     &|a| {
    //         let buffer:[u8;1] = [a as u8];
    //         stderr().lock().write(&buffer).expect("Write Error") as i64;
    //         2i64            
    //     }, //STDERR WRITE BYTE
    // ];

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
            b'?' => { //READ BYTE stack: [fp] -> [char]
                // let fp = stack.pop().oos(&strict);

                let mut buffer:[u8;1] = [0];
                let stack_value = match stdin().lock().read_exact(&mut buffer){
                    Ok(_) => buffer[0] as i64,
                    Err(_) => -1,   
                };
                stack.push(stack_value);

                // match fp{
                //     0 => {
                //         let mut buffer:[u8;1] = [0];
                //         let stack_value = match stdin().lock().read_exact(&mut buffer){
                //             Ok(_) => buffer[0] as i64,
                //             Err(_) => -1,   
                //         };
                //         stack.push(stack_value);
                //     }
                //     _ => {
                //         if strict {
                //             eprintln!("Strict mode violation: Wrong fd for READ, for now only 0 STDIN is implemented!");
                //             exit(1);
                //         }
                //     }
                // }
            }
            b'.' => { //WRITE BYTE stack: [ch, fp] -> []
                // let fp = stack.pop().oos(&strict);
                let ch = stack.pop().oos(&strict);

                let buffer:[u8;1] = [ch as u8];
                let _ = stdout().lock().write(&buffer);
                // stack.push(
                //     match ret{
                //         Ok(_) => 0i64,
                //         Err(_) => -1i64,
                //     }
                // )

                // match fp{
                //     1 => {
                //         let buffer:[u8;1] = [ch as u8];
                //         let ret = stdout().lock().write(&buffer);
                //         stack.push(
                //             match ret{
                //                 Ok(_) => 0i64,
                //                 Err(_) => -1i64,
                //             }
                //         )
                //     }
                //     2 =>{
                //         let buffer:[u8;1] = [ch as u8];
                //         let ret = stderr().lock().write(&buffer);
                //         stack.push(
                //             match ret{
                //                 Ok(_) => 0i64,
                //                 Err(_) => -1i64,
                //             }
                //         )
                //     }
                //     _ => {
                //         if strict {
                //             eprintln!("Strict mode violation: Wrong fd for WRITE, for now only 1 STDOUT and 2 STDERR is implemented!");
                //             exit(1);
                //         }                
                //     }
                // }
            }            
            _ => {
                panic!("INTERPRETER's FAULT: Invalid token!")
            }
        }

        index = (Wrapping(index)+Wrapping(1usize)).0;
        if (index<0) | (index >= code.len()) {break}
    }

    if debug{
        // eprintln!("{:?}", &stack);
        eprintln!("{:?}", &stack);
        stack = &mut stacks[!stack_index&1];
        eprintln!("{:?}", &stack);
    }
    

}
