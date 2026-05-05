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
.entry:
    xor rax, rax
    mov rsp, rbp
    pop rbp
    ret

