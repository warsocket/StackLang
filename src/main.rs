use std::fs::File;
use std::env;
use std::process::exit;
use std::num::Wrapping;
use std::arch::asm;
use std::io::{Read, Write, stdin, stdout, stderr};
use std::collections::HashMap;

const TOKENS:[u8;16] = [ //encoding 1 nibble pertoken, 2 per byte
    b'(', // PUSH(reg)
    b')', // POP to reg
    b'+', // ADD
    b'-', // SUB
    b'*', // MUL (returns 2 stack numbners)
    b'/', // DIV
    b'|', // OR
    b'&', // AND
    b'^', // XOR
    b'!', // NOT
    b'1', // SHL 1; | 1
    b'0', // SHL 1
    b'$', // FUNCTION
    b'@', // JUMP (0 == NOP)
    b'=', // TEST (set flags for greater smaller etc), 0 result = same
    b':', // SWITCH TO STACK #
]; //Jumping outsid eof the array of instruction = HALT

fn main() {

    let mut args = env::args();
    args.next();

    let mut debug = false;
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
                "--debug" => {
                    debug = true;
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
            run(&tokenise(script_bytes), debug); 
        },
        Mode::COMPILE => { 
            let mut out = stdout().lock();
            out.write( &compile(&tokenise(script_bytes)) ); 
        },
        Mode::BYTECODE => { 
            run(&bytecode(&script_bytes), debug); 
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

const STACK_ZERO:&str = "Error: Stack Depleted!";


trait Oos{
    fn oos(self:&Self)->i64;
}

impl Oos for Option<i64>{
    fn oos(self:&Self) -> i64{
        match(self){
            Some(v) => *v,
            None => {
                eprintln!("{}", STACK_ZERO);
                exit(1);
            }
        }

    }    
}



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
            eprintln!("{:#018X}: {}", index, code[index] as char);
        }

        match(code[index]){
            b'(' => {
                stack.push(reg);
            }
            b')' => {
                reg = stack.pop().oos();
            }
            b'+' => {
                let b = stack.pop().oos();
                let a = stack.pop().oos();
                stack.push( (Wrapping(a)+Wrapping(b)).0 );
            }
            b'-' => {
                let b = stack.pop().oos();
                let a = stack.pop().oos();
                stack.push( (Wrapping(a)-Wrapping(b)).0 );
            }
            b'*' => {
                let b = stack.pop().oos();
                let a = stack.pop().oos();
                let x = i128::from(a)*i128::from(b);
                stack.push((x >> 64) as i64);
                stack.push(x as i64);
            }
            b'/' => {
                let b = stack.pop().oos();
                let a = stack.pop().oos();
                stack.push(a/b);
            }
            b'|' => {
                let b = stack.pop().oos();
                let a = stack.pop().oos();
                stack.push( a | b );
            }
            b'&' => {
                let b = stack.pop().oos();
                let a = stack.pop().oos();
                stack.push( a & b );
            }
            b'^' => {
                let b = stack.pop().oos();
                let a = stack.pop().oos();
                stack.push( a ^ b );
            }
            b'!' => {
                let a = stack.pop().oos();
                stack.push( !a );
            }
            b'1' => {
                let a = stack.pop().oos();
                stack.push((a << 1) | 0b1);
            }
            b'0' => {
                let a = stack.pop().oos();
                stack.push((a << 1));
            }
            b'$' => {
                let a = stack.pop().oos();
                stack.push( func_matrix[reg as usize](a) );
            }
            b'@' => {
                //JMP 0 = NOP and advances 1
                let a = stack.pop().oos();
                if a != 0{
                    step = false;    
                    index = a as usize + index;
                }
            }
            b'=' => {
                // last 2 bits contain flags [negative, non-zero]
                let a = stack.pop().oos();
                stack.push( ((a<0) as i64) <<1 | (a==0) as i64 )
            }
            b':' => {
                let a = stack.pop().oos();
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
