global main
extern printf

section .rodata
msg: db "Hello World!", 13, 10, 0

section .text
main:
  sub  rsp,   8
  sub  rsp,   32
  mov  rcx,   qword msg
  call printf
  add  rsp,   32

  xor eax, eax
  add rsp, 8
  ret
