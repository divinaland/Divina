global main
extern library_perform

section .text
main:
  call library_perform

  xor eax, eax
  add rsp, 8
  ret
