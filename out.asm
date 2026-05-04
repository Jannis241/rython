section .text
global _start

_start:
    call main
    mov rdi, rax
    mov rax, 60
    syscall

test:
    push rbp
    mov rbp, rsp
    sub rsp, 32
.entry:
    lea rax, [rbp - 32]
    mov [rbp - 8], rax
    mov rax, 5
    mov [rbp - 16], rax
    mov rax, [rbp - 8]
    mov rcx, [rbp - 16]
    mov [rax], rcx
    mov rax, [rbp - 8]
    mov rcx, [rax]
    mov [rbp - 24], rcx
    mov rax, [rbp - 24]
    mov rsp, rbp
    pop rbp
    ret

