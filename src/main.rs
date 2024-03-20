use std::fs::File;
use std::env;
use std::process::exit;
use std::num::Wrapping;
use std::arch::asm;
use std::io::{Read, Write, stdin, stdout, stderr};
use std::collections::HashMap;


fn main() {

    let mut args = env::args();
    args.next();

    let mut debug = false;
    let params:Vec<String> = args.collect();
    let mut filename = "".to_owned();

    // println!("{}", fil)

    for p in params{
        let param = p.as_str();

        if param.starts_with("--"){
            match(param){
                "--debug" => {
                    debug = true;
                },
                "--compile" => {
                    todo!("comile to bytecode");
                },
                "--bytecode" => {
                    todo!("run bytecode");
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

    let mut pure_script:Vec<u8> = vec!();
    const valid_tokens:[u8;16] = [ //encoding 1 nibble pertoken, 2 per byte
        b'(', // PUSH(reg)
        b')', // POP to reg
        b'+', // ADD
        b'-', // SUB
        b'*', // MUL
        b'/', // DIV
        b'|', // OR
        b'&', // AND
        b'^', // XOR
        b'!', // NOT
        b'1', // SHL 1; | 1
        b'0', // SHL 1
        b'$', // FUNCTION
        b'@', // JUMP
        b'=', // TEST (set flags for greater smaller etc), 0 result = same
        b':', // SWITCH TO STACK #
    ]; //Jumping outsid eof the array of instruction = HALT

    let mut ignore_to_newline = false;
    for token in script_bytes{

        if ignore_to_newline{
            if token == b'\n'{ ignore_to_newline = false; }
        }else{
            if valid_tokens.contains(&token){
                pure_script.push(token)
            }else if token == b'#'{
                ignore_to_newline = true;
            }            
        }
    }

    run(&pure_script, debug)

}

const STACK_ZERO:&str = "Error: Stack Empty.";

fn run(code:&Vec<u8>, debug:bool){

    if code.len() == 0{return}

    let mut reg:i64 = 0;
    let mut index = 0;
    let mut step = true;

    let mut stackmap:HashMap<i64, Vec<i64>> = HashMap::new();
    let mut stackindex:i64 = 0;

    stackmap.insert(stackindex, vec!());
 


    let func_matrix:[&dyn Fn(i64) -> i64;3] = [
        &|a| {
        let mut buffer:[u8;1] = [0];
        stdin().lock().read_exact(&mut buffer);
        buffer[0] as i64
        }, //STDIN READ BYTE
        &|a| {
            let buffer:[u8;1] = [a as u8];
            stdout().lock().write(&buffer).expect("Write Error") as i64;
            1i64 //
        }, //STDOUT WRITE BYTE
        &|a| {
            let buffer:[u8;1] = [a as u8];
            stderr().lock().write(&buffer).expect("Write Error") as i64;
            2i64            
        }, //STDERR WRITE BYTE
    ];

    loop{
        let stack: &mut Vec<i64> = stackmap.get_mut(&stackindex).expect("INTERPRETER's FAULT: Stackindex should always be valid!");
        step = true; //all stuff steps except for one so we just set it boforehand

        if debug{
            eprintln!("{}{:?} reg={}", stackindex, &stack, reg);
            eprintln!("{}", code[index] as char);
        }

        match(code[index]){
            b'(' => {
                stack.push(reg);
            }
            b')' => {
                reg = stack.pop().expect(STACK_ZERO);
            }
            b'+' => {
                let b = stack.pop().expect(STACK_ZERO);
                let a = stack.pop().expect(STACK_ZERO);
                stack.push( (Wrapping(a)+Wrapping(b)).0 );
            }
            b'-' => {
                let b = stack.pop().expect(STACK_ZERO);
                let a = stack.pop().expect(STACK_ZERO);
                stack.push( (Wrapping(a)-Wrapping(b)).0 );
            }
            b'*' => {
                let b = stack.pop().expect(STACK_ZERO);
                let a = stack.pop().expect(STACK_ZERO);
                let x = i128::from(a)*i128::from(b);
                stack.push((x >> 64) as i64);
                stack.push(x as i64);
            }
            b'/' => {
                let b = stack.pop().expect(STACK_ZERO);
                let a = stack.pop().expect(STACK_ZERO);
                let x = i128::from(a)/i128::from(b);
                stack.push((x >> 64) as i64);
                stack.push(x as i64);
            }
            b'|' => {
                let b = stack.pop().expect(STACK_ZERO);
                let a = stack.pop().expect(STACK_ZERO);
                stack.push( a | b );
            }
            b'&' => {
                let b = stack.pop().expect(STACK_ZERO);
                let a = stack.pop().expect(STACK_ZERO);
                stack.push( a & b );
            }
            b'^' => {
                let b = stack.pop().expect(STACK_ZERO);
                let a = stack.pop().expect(STACK_ZERO);
                stack.push( a ^ b );
            }
            b'!' => {
                let a = stack.pop().expect(STACK_ZERO);
                stack.push( !a );
            }
            b'1' => {
                let a = stack.pop().expect(STACK_ZERO);
                stack.push((a << 1) | 0b1);
            }
            b'0' => {
                let a = stack.pop().expect(STACK_ZERO);
                stack.push((a << 1));
            }
            b'$' => {
                let a = stack.pop().expect(STACK_ZERO);
                stack.push( func_matrix[reg as usize](a) );
            }
            b'@' => {
                let a = stack.pop().expect(STACK_ZERO);
                index = a as usize;
                step = false;
            }
            b'=' => {
                // last 2 bits contain flags [negative, non-zero]
                let a = stack.pop().expect(STACK_ZERO);
                stack.push( ((a<0) as i64) <<1 | (a==0) as i64 )
            }
            b':' => {
                let a = stack.pop().expect(STACK_ZERO);
                if stack.len() == 0 { stackmap.remove(&stackindex); } //prune empty maps you leave

                if !stackmap.contains_key(&a){
                    stackmap.insert(a, vec!());
                }
                stackindex = a;
            }            
            _ => {
                panic!("INTERPRETER's FAULT: Invalid token!")
            }
        }

        if step {index += 1};
        if (index<0) | (index >= code.len()) {break}
    }

    if debug{eprintln!("{}{:?} reg={}", stackindex, stackmap.get_mut(&stackindex).unwrap(), reg);}
    

}
