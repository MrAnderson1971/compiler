.global main
main:
pushq %rbp
movq %rsp, %rbp
subq $16, %rsp
movl $0, -4(%rbp)
movl $1, %r10d
movl %r10d, -8(%rbp)
addl $2, -8(%rbp)
movl $1, %r10d
movl %r10d, -12(%rbp)
addl $2, -12(%rbp)
movl -12(%rbp), %r10d
movl %r10d, -4(%rbp)
movl -4(%rbp), %eax
movq %rbp, %rsp
popq %rbp
ret
