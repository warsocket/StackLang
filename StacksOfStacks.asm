; See https://upload.wikimedia.org/wikipedia/commons/e/e4/ELF_Executable_and_Linkable_Format_diagram_by_Ange_Albertini.png
; Assemble using: nasm -f bin
[bits 64]

; Virtual address of where the ELF file will be mapped
OFFSET: equ 0x10000

; ELF header
db 0x7f, "ELF" ; Magic number
db 2 ; class =  64 bit
db 1 ; little endian
db 1 ; version = 1
db 0 ; ABI
db 0 ; extended ABI
times 7 db 0 ; Padding
dw 2 ; type = executable
dw 0x3e ; arch = x86_64
dd 1 ; version = 1
dq entry_point + OFFSET ; Entry point in memmory

dq program_headers ; program header offset
dq 0 ; section_headers_start
dd 0 ; other flags
; Size of this header, 64 bytes.
dw program_headers; size of this header
; Size of a program header entry.
dw program_headers; size of program entry header
; Number of program header entries.
; dw 0 ; # of program entry headers
; dw 0x40-2 ; size of section entry header
; dw 0 ; # of section headers
; dw 0 ; index of section header with string table



program_headers:
dd 1 ; = loadable srgment
dd 5; Flags: 0x01 = executable, 0x02 = writable, 0x04 = readable
dq 0 ; loadable segment offset (load everything from start)
dq OFFSET ; Virutal address where to palce this elf in memory
dq OFFSET ; Physical address where to palce this elf in memory (seems unsused in x86_64)
dq code_section_end ; siz eof segment in the elf file
dq code_section_end ; size of segment in memmory
; dq 0;0x200000 ; segment alignment (seems ignored)

; Program starts here
STACKSIZE equ 0x10000000; 256MB

; Actual code
entry_point:

; Setup 36 bytes
mov rax, 12
xor rdi, rdi
syscall

mov rbp, STACKSIZE
add rbp, rax

mov rdi, STACKSIZE*2
add rdi, rax
mov rax, 12

syscall

mov rsp, rax ; now RSP points to active stack and RBP to the inactive one

; xxx:
; mov rax , xxx-entry_point
; ; code shoud go here
; mov rax, OFFSET+code_section_end-10 ;this jumps to the exit syscall
; jmp rax

; ! PUSH -1
db "__!__"
push -1
db "_____"

; ^ XOR
db "__^__"
pop rcx
xor [rsp], rcx
db "_____"

; | OR
db "__|__"
pop rcx
or [rsp], rcx
db "_____"

; & AND
db "__&__"
pop rcx
and [rsp], rcx
db "_____"


; + ADD
db "__+__"
pop rcx
add [rsp], rcx
db "_____"

; - SUB
db "__-__"
pop rcx
sub [rsp], rcx
db "_____"

; * MUL
db "__*__"
pop rax
pop rcx
mul rcx
push rax
db "_____"

; / DIV
db "__/__"
xor rdx, rdx
pop rcx
pop rax
test rcx, rcx
jz short $+5 ;jumps over div rcx
div rcx
cmovz rax, rdx ; result  = 0 if division by zero
push rax
db "_____"

; $ STACK SWAP
db "__$__"
xchg rbp, rsp
db "_____"

; ~ XCHANGE
db "__~__"
pop rax
xchg rax, [rbp]
push rax
db "_____"

; = DUP
db "__=__"
mov rax, [rsp]
push rax
db "_____"

; ? READ
db "__?__"
lea rsp, [rsp-8]
xor rax, rax
xor rdi, rdi
mov rsi, rsp
mov rdx, 1
syscall
db "_____"

; . WRITE
db "__.__"
mov rax, 1
mov rsi, rsp
mov rdi, rax
mov rdx, rax
lea rsp, [rsp+8]
syscall
db "_____"

; 0 SHL0
db "__0__"
pop rax
shl rax, 1
push rax
db "_____"

; 1 SHL1
db "__1__"
pop rax
shl rax, 1
or al, 1
push rax
db "_____"


;Exit 10 bytes
mov rax, 60
xor rdi, rdi
syscall

code_section_end:
; code_section_end equ "%ASMEND%"
; code_section_end: OFFSET + sizeof(code)