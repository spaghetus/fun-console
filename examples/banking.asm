* = $0200
.text "#!fun"
.byte $0A
.text "BANKING   "
.fill 240, [$00]
entry
	lda #0
	sta $4010
	jsr print
	lda #1
	sta $4010
	jsr print
	jmp finish

print
	ldx #0
loop
	lda $2000,x
	inx
	sta $4019
	ADC #0
	bne loop
	rts

finish
	jmp finish