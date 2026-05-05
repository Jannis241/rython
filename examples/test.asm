section .data


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
.entry:
    
        mov rax, 1
        mov rdi, 1
        mov rsi, msg
        mov rdx, len
        syscall
    
    xor rax, rax
    mov rsp, rbp
    pop rbp
    ret

