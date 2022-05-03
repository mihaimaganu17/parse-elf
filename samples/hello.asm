[BITS 64]
; Linux calling convention
; 1. rdi
; 2. rsi
; 3. rdx
; 4. rcx
; 5. r8
; 6. r9

global _start

section .text

_start:
    mov rdi, 1  ; stdout handle
    mov rsi, msg ; load "hi there" in rdx
    mov rdx, 9  ; length for msg
    mov rax, 1  ; write syscall
    syscall
    mov rdi, 0; ; exit code
    mov rax, 60 ; exit syscall
    syscall

section .data

msg db "hi there", 10, 13, 0
