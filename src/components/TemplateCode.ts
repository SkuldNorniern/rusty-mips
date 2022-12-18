interface Code {
  name: string
  code: string
}

export const TEMPLATE_CODE: Code[] = [
  {
    name: '재귀 피보나치',
    code: `
# Recursive fibonacci calculator
# Function signature: int fibonacci(int)
# Also saves the result into $gp as an int array (e.g. $gp = fibonacci(2), $gp + 4 = fibonacci(3), ...)
# Modified from https://gist.github.com/libertylocked/068b118354539a8be992
.text
.globl main
main:
    # Calculate fibonacci upto this number (7)
    ori $a0, $0, 7
    or $s0, $ra, $zero
    jal fibonacci

    # Now the answer is in $v0
    # NOP here so you can check out register pane
    nop

    or $ra, $s0, $zero
    # Terminate the program
    jr $ra
fibonacci:
    # Prologue
    addi $sp, $sp, -12
    sw $ra, 8($sp)
    sw $s0, 4($sp)
    sw $s1, 0($sp)
    or $s0, $a0, $zero
    ori $v0, $zero, 1 # return value for terminal condition
    sw $0, 0($gp)
    sw $v0, 4($gp)
    slti $t0, $16, 3
    bne $t0, $0, fibonacciExit # check terminal condition
    addi $a0, $s0, -1 # set args for recursive call to f(n-1)
    jal fibonacci
    or $s1, $v0, $zero # store result of f(n-1) to s1
    addi $a0, $s0, -2 # set args for recursive call to f(n-2)
    jal fibonacci
    add $v0, $s1, $v0 # add result of f(n-1) to it
fibonacciExit:
    # Save value to memory
    add $t0, $s0, $s0
    add $t0, $t0, $t0  # multiply by 4
    add $t0, $gp, $t0
    sw $v0, 0($t0)
    # Epilogue
    lw $ra, 8($sp)
    lw $s0, 4($sp)
    lw $s1, 0($sp)
    addi $sp, $sp, 12
    jr $ra`
  },
  {
    name: '간단한 덧셈',
    code: `
.text
.globl main
main:
    addi $s0, $0, 1
    addi $s1, $s0, 1
    add $s2, $s0, $s1`
  },
  {
    name: '반복문 피보나치',
    code: `
.text
.globl main
main:
    # 아래 숫자 (7) 까지의 피보나치 숫자를 구함
    # (결과로 0xd가 나와야 함)
    addi $a0, $0, 7
    j fibonacci
fibonacci:
    addi $t0, $0, 0
    addi $t1, $0, 1
    addi $t2, $a0, -1
    sw $t0, 0($gp)
    sw $t1, 4($gp)
    addi $gp, $gp, 8
loop:
    add $t3, $t0, $t1
    or $t0, $0, $t1
    or $t1, $0, $t3
    sw $t3, 0($gp)
    addi $gp, $gp, 4
    addi $t2, $t2, -1
    beq $t2, $0, endFibonacci
    j loop
endFibonacci:
    or $v0, $0, $t1
    j 0x00000000`
  },
  {
    name: '버블정렬',
    code: `
.data 0x10008000
.word 5 1 4 2 3

.text
.globl main
main:
    or      $a0, $0, $gp
    addi    $a1, $0, 5

    j       bubblesort

bubblesort:  # prototype: void bubblesort(int* arr, int len)
    addi    $15, $0, 1
    slt     $15, $5, $15
    beq     $15, $0, .L8

    j       .L1
.L8:
    addi    $2, $0, 1
    beq     $5, $2, .L1

    sll     $7, $5, 2
    add     $7, $4, $7
    or      $9, $0, $0
    addi    $10, $4, 4
.L6:
    or      $2, $10, $0
    or      $8, $0, $0
.L4:
    lw      $3, -4($2)
    lw      $4, 0($2)
    nop
    slt     $6, $4, $3
    beq     $6, $0, .L3

    sw      $4, -4($2)
    sw      $3, 0($2)
    addi    $8, $0, 1
.L3:
    addi    $2, $2, 4
    beq     $7, $2, .L7

    j       .L4
.L7:
    beq     $8, $0, .L1

    addi    $9, $9, 1
    beq     $5, $9, .L1

    j       .L6

.L1:
    j       0x00000000`
  }
];
