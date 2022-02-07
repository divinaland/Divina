global library_perform
extern printf

section .rodata
message: db "Hello World!", 13, 10, 0

section .text
library_perform:
  sub  rsp,   8
  sub  rsp,   32
  mov  rcx,   qword message
  call printf
  add  rsp,   32

  ret
