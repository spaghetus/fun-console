* = $0200
.text "#!fun"
.byte $0A
.text "COOL GAME "
.fill 240, [$00]
entry
	jmp print
text
	.null "This is a very cool game!"
print
	ldx #0
loop
	lda text,x
	inx
	sta $4019
	ADC #0
	bne loop
finish
	jmp finish