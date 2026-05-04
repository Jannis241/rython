    
    section .data:
        msg db "Jannis Der Eier Kopf Yessirsky", 10
        len equ $ - msg


section .text
global _start

_start:
    call main
    mov rdi, rax
    mov rax, 60
    syscall

main:
    push rbp
    mov rbp, rsp
    sub rsp, 32
.entry:
    lea rax, [rbp - 24]
    mov [rbp - 8], rax
    mov rax, 100
    mov [rbp - 16], rax
    mov rax, [rbp - 8]
    mov rcx, [rbp - 16]
    mov [rax], rcx
    
        mov [rbp - 16], rcx    ; store value on stack
        lea rsi, [rbp - 16]    ; rsi = address of that value
        mov rax, 1      ; syscall: write
        mov rdi, 1      ; fd: stdout
        mov rdx, 2    ; length
        syscall
    
    
        mov rax, 1
        mov rdi, 1
        mov rsi, msg
        mov rdx, len
        syscall
    
    xor rax, rax
    mov rsp, rbp
    pop rbp
    ret

